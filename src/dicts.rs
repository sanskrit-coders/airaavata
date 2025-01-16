use indicatif::ProgressBar;
use log::{error, info};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use vidyut_kosha::{Kosha};
use vidyut_kosha::entries::{BasicPratipadikaEntry, PratipadikaEntry};
use vidyut_lipi::{transliterate, Scheme};
use vidyut_prakriya::args::{BaseKrt, Dhatu, DhatuPada, Gana, Krdanta, Krt, Lakara, Pada, Pratipadika, Prayoga, Purusha, Sanadi, Subanta, Taddhita, Taddhitanta, Tinanta, Vacana, Vibhakti};
use vidyut_prakriya::{Dhatupatha, Vyakarana};

use crate::util::{dev, slp};

// Equivalent to OrderedSet - using HashSet for now, can be replaced with IndexSet if ordering is critical
type OrderedSet<T> = HashSet<T>;

lazy_static::lazy_static! {
    static ref SANAADI_DICT_KRDANTA: HashMap<&'static str, Vec<Sanadi>> = {
        let mut m = HashMap::new();
        m.insert("vidyut-kRdanta", vec![]);
        m.insert("vidyut-Nic-kRdanta", vec![Sanadi::Ric]);
        m.insert("vidyut-san-kRdanta", vec![Sanadi::san]);
        m.insert("vidyut-yaN-kRdanta", vec![Sanadi::yaN]);
        m.insert("vidyut-yaNluk-kRdanta", vec![Sanadi::yaNluk]);
        m.insert("vidyut-san-Nic-kRdanta", vec![Sanadi::san, Sanadi::Ric]);
        m.insert("vidyut-Nic-san-kRdanta", vec![Sanadi::Ric, Sanadi::san]);
        m
    };

    static ref SANAADI_DICT_TINANTA: HashMap<&'static str, Vec<Sanadi>> = {
        let mut m = HashMap::new();
        m.insert("vidyut-tiN", vec![]);
        m.insert("vidyut-Nic-tiN", vec![Sanadi::Ric]);
        m.insert("vidyut-san-tiN", vec![Sanadi::san]);
        m.insert("vidyut-yaN-tiN", vec![Sanadi::yaN]);
        m.insert("vidyut-yaN-luk-tiN", vec![Sanadi::yaNluk]);
        m.insert("vidyut-san-Nic-tiN", vec![Sanadi::san, Sanadi::Ric]);
        m.insert("vidyut-Nic-san-tiN", vec![Sanadi::Ric, Sanadi::san]);
        m
    };
}

#[derive(Debug, Serialize, Deserialize)]
struct Definition {
    headwords: Vec<String>,
    meaning: String,
}

struct BabylonDictionary {
    v: Vyakarana,
    kosha: Kosha
}

impl BabylonDictionary {
    fn new(kosha_path: &Path) -> Self {
        let kosha = Kosha::new(kosha_path).unwrap();
        let v = Vyakarana::new();

        Self { v, kosha }
    }


    fn get_krdanta_entry(
        &self,
        entry_head: String,
        mut headwords_in: OrderedSet<String>,
        sanaadyanta: Dhatu, prayoga: Prayoga
    ) -> Vec<Definition> {
        let mut entry = format!("{}<BR>", entry_head);

        for krt in BaseKrt::iter() {
            let anga = Krdanta::builder().dhatu(sanaadyanta.clone()).krt(krt).build();
            let prakriyas = self.v.derive_krdantas(&anga.unwrap());

            for p in prakriyas {
                headwords_in.insert(dev(p.text()));
                entry.push_str(&format!(
                    "{}+{} = {}<BR>",
                    dev("+"),
                    dev(&krt.to_string()),
                    dev(p.text())
                ));
            }
        }

        vec![Definition {
            headwords: headwords_in.into_iter().collect(),
            meaning: entry,
        }]
    }

    fn get_tinanta_entry(
        &self,
        entry_head: String,
        headwords_in: OrderedSet<String>,
        sanaadyanta: Dhatu,
        prayoga: Prayoga,
    ) -> Vec<Definition> {
        let mut definitions = Vec::new();

        for lakara in Lakara::iter() {
            let mut headwords = Vec::new();
            let mut table_lines = Vec::new();

            for parasmai_mode in &[DhatuPada::Parasmaipada, DhatuPada::Atmanepada] {
                let mut lines = Vec::new();
                let mut pada_headwords = Vec::new();

                for purusha in Purusha::iter() {
                    let mut vacana_forms = Vec::new();

                    for vacana in Vacana::iter() {
                        let pada = Tinanta::builder().dhatu(sanaadyanta.clone()).prayoga(prayoga).pada(*parasmai_mode).lakara(lakara).purusha(purusha).vacana(vacana).build();

                        let prakriyas = self.v.derive_tinantas(&pada.unwrap());
                        let forms: Vec<String> =
                            prakriyas.iter().map(|p| dev(p.text())).collect();

                        pada_headwords.extend(forms.clone());
                        vacana_forms.push(forms.join("/ "));
                    }

                    let purusha_line = vacana_forms.join("<BR>");
                    lines.push(purusha_line);
                }

                if !pada_headwords.is_empty() {
                    let mut table_head =
                        format!("{} {}", entry_head, dev(&lakara.to_string()));
                    if prayoga == Prayoga::Karmani {
                        table_head.push_str(" अकर्तरि<BR><BR>");
                    } else {
                        table_head.push_str(&format!(" {}", dev(&parasmai_mode.to_string())));
                    }
                    table_lines.push(table_head);
                    table_lines.push(lines.join("<BR>--<BR>"));
                    headwords.extend(pada_headwords);
                }
            }

            if !headwords.is_empty() {
                let mut all_headwords = headwords_in.clone();
                all_headwords.extend(headwords.into_iter());

                let mut entry = table_lines.join("<BR><BR>");
                entry = entry.replace("लृँत्", "लृँट्");

                definitions.push(Definition {
                    headwords: all_headwords.into_iter().collect(),
                    meaning: entry,
                });
            }
        }

        definitions
    }

    fn dump_subantas(&self, dest_dir: &Path) {
        let dicts: HashMap<&str, (&str, &str)> = [
            ("a", ("", "इ")),
            ("i", ("इ", "उ")),
            ("uch", ("उ", "क")),
            ("ku", ("क", "च")),
            ("chu", ("च", "ट")),
            ("Tu", ("ट", "त")),
            ("tu1", ("त", "प")),
            ("pu", ("प", "य")),
            ("yrlv", ("य", "श")),
            ("shal", ("श", "ा")),
        ]
            .into_iter()
            .collect();

        for (dict_name, (border_start, border_end)) in dicts {
            let mut definitions = Vec::new();
            let dict_name = format!("vidyut-subanta-{}", dict_name);

            let progress_bar = ProgressBar::new_spinner();
            progress_bar.set_message(format!("Processing {}", dict_name));

            for praatipadika in self.kosha.pratipadikas() {
                if matches!(praatipadika, PratipadikaEntry::Krdanta(_)) {
                    continue;
                }
                let basic_pratipadika = match praatipadika {
                    PratipadikaEntry::Basic(basic) => basic,
                    _ => panic!("Expected BasicPratipadika")
                };                
                
                let praatipadika_str = dev(&basic_pratipadika.pratipadika().text());
                if !(praatipadika_str >= border_start.to_string()
                    && praatipadika_str < border_end.to_string())
                {
                    continue;
                }

                for linga in basic_pratipadika.lingas() {
                    let mut headwords = OrderedSet::new();
                    headwords.insert(praatipadika_str.clone());
                    let mut lines = Vec::new();

                    for vibhakti in Vibhakti::iter() {
                        let mut vachana_entries = Vec::new();
                        for vacana in Vacana::iter() {
                            let pada = Subanta::builder().pratipadika(basic_pratipadika.clone()).linga(linga).vibhakti(vibhakti).vacana(vacana).build();

                            let prakriyas = self.v.derive_subantas(pada);
                            let mut forms = Vec::new();

                            for prakriya in prakriyas {
                                let pada_str = dev(prakriya.text());
                                headwords.insert(pada_str.clone());
                                forms.push(pada_str);
                            }

                            let vachana_entry = forms.join(", ");
                            vachana_entries.push(vachana_entry);
                        };
                        lines.push(vachana_entries.join("; "));
                    }
                    let linga_str = dev(&linga.to_string());
                    let meaning = format!(
                        "{} {}<BR>{}",
                        praatipadika_str,
                        &linga_str[..4.min(linga_str.len())],
                        lines.join("<BR>")
                    );

                    definitions.push(Definition {
                        headwords: headwords.into_iter().collect(),
                        meaning,
                    });
                }

                progress_bar.tick();
            }

            progress_bar.finish_with_message(format!(
                "Got {} definitions for {}",
                definitions.len(),
                dict_name
            ));

            let dest_file_path = dest_dir
                .join(&dict_name)
                .join(format!("{}.babylon", dict_name));
            self.dump_babylon(&dest_file_path, &definitions);
        }

    }

    fn dump_taddhitaantas(&self, dest_dir: &Path, overwrite: bool) {
        let dicts: HashMap<&str, (&str, &str)> = [
            ("a", ("", "इ")),
            ("i", ("इ", "उ")),
            ("uch", ("उ", "क")),
            ("ku", ("क", "च")),
            ("chu", ("च", "ट")),
            ("Tu", ("ट", "त")),
            ("tu1", ("त", "प")),
            ("p", ("प", "ब")),
            ("b", ("ब", "य")),
            ("yr", ("य", "ल")),
            ("lv", ("ल", "व")),
            ("sh", ("श", "स")),
            ("s", ("स", "ह")),
            ("hal", ("ह", "ा")),
        ]
            .into_iter()
            .collect();

        for (dict_name, (border_start, border_end)) in dicts {
            let dict_name = format!("vidyut-taddhitAnta-{}", dict_name);
            let dest_file_path = dest_dir
                .join(&dict_name)
                .join(format!("{}.babylon", dict_name));

            if !overwrite && dest_file_path.exists() {
                info!("Skipping {}", dict_name);
                continue;
            }

            info!("Producing {}", dict_name);
            let mut definitions = Vec::new();

            let progress_bar = ProgressBar::new_spinner();
            progress_bar.set_message(format!("Processing {}", dict_name));

            for praatipadika in self.kosha.pratipadikas() {
                if matches!(praatipadika, PratipadikaEntry::Krdanta(_)) {
                    continue;
                }
                let basic_pratipadika = match praatipadika {
                    PratipadikaEntry::Basic(basic) => basic,
                    _ => panic!("Expected BasicPratipadika")
                };

                let praatipadika_str = dev(&basic_pratipadika.pratipadika().text());
                if !(praatipadika_str >= border_start.to_string()
                    && praatipadika_str < border_end.to_string())
                {
                    continue;
                }

                let mut headwords = OrderedSet::new();
                headwords.insert(praatipadika_str.clone());
                let mut lines = Vec::new();

                for taddhita in Taddhita::iter() {
                    let anga =
                        Taddhitanta::builder().pratipadika(praatipadika.clone()). taddhita(taddhita).build();
                    let prakriyas = self.v.derive_taddhitantas(anga);

                    if !prakriyas.is_empty() {
                        let derivatives: Vec<String> =
                            prakriyas.iter().map(|p| dev(p.text())).collect();

                        headwords.extend(derivatives.clone());
                        lines.push(format!(
                            "+ {} = {}",
                            dev(&taddhita.to_string()),
                            derivatives.join(", ")
                        ));
                    }
                }

                let linga_str = dev(
                    &basic_pratipadika
                        .lingas()
                        .iter()
                        .map(|l| l.to_string())
                        .collect::<Vec<_>>()
                        .join(","),
                );

                let meaning = format!(
                    "{} {}<BR>{}",
                    praatipadika_str,
                    linga_str,
                    lines.join("<BR>")
                );

                definitions.push(Definition {
                    headwords: headwords.into_iter().collect(),
                    meaning,
                });

                progress_bar.tick();
            }

            progress_bar.finish_with_message(format!("Got {} definitions", definitions.len()));
            self.dump_babylon(&dest_file_path, &definitions);
        }
    }

    fn dump_sanaadi_dicts(
        &self,
        dest_dir: &Path,
        sanaadi_dict: &HashMap<&str, Vec<Sanadi>>,
        make_entry: fn(&BabylonDictionary, String, OrderedSet<String>, Dhatu, Prayoga) -> Vec<Definition>,
    ) {
        let dhatu_entries = self.kosha.dhatus();

        for (dict_name, sanadi) in sanaadi_dict {
            let prayogas = if sanaadi_dict == &*SANAADI_DICT_KRDANTA {
                vec![Prayoga::Kartari]
            } else {
                vec![Prayoga::Kartari, Prayoga::Karmani]
            };

            for prayoga in prayogas {
                let prayoga_suffix = if prayoga == Prayoga::Kartari {
                    ""
                } else {
                    "-akartari"
                };

                let dict_name = format!("{}{}", dict_name, prayoga_suffix);
                let mut definitions = Vec::new();

                let progress_bar = ProgressBar::new(dhatu_entries.len() as u64);
                progress_bar.set_message(format!("Dhaatus {}", dict_name));

                for dhatu_entry in dhatu_entries.iter() {
                    let mut headwords_in = OrderedSet::new();
                    let aupadeshika = dev(&dhatu_entry.dhatu.aupadeshika);

                    // Add variations of aupadeshika
                    headwords_in.insert(aupadeshika.clone());
                    headwords_in.insert(
                        regex::Regex::new("[॒॑]")
                            .unwrap()
                            .replace_all(&aupadeshika, "")
                            .to_string(),
                    );
                    headwords_in.insert(
                        regex::Regex::new("[॒॑ँ]")
                            .unwrap()
                            .replace_all(&aupadeshika, "")
                            .to_string(),
                    );

                    let mut dhatu_str = format!(
                        "{} {} ({})",
                        dhatu_entry.dhatu.aupadeshika, dhatu_entry.artha, dhatu_entry.dhatu.gana
                    );

                    for p in self.v.derive_tinantas(dhatu_entry.dhatu.clone()) {
                        let dhatu_form = dev(p.text());
                        if dev(&dhatu_entry.dhatu.aupadeshika) != dhatu_form {
                            dhatu_str.push_str(&format!(" {}", dhatu_form));
                            headwords_in.insert(dhatu_form);
                        }
                    }

                    if let Some(antargana) = &dhatu_entry.dhatu.antargana {
                        dhatu_str.push_str(&format!(" ({})", antargana));
                    }

                    let sanaadyanta = dhatu_entry.dhatu.with_sanadi(sanadi);
                    let mut sanaadi_str = String::new();

                    for p in self.v.derive_tinantas(sanaadyanta.clone()) {
                        let sanaadyanta_str = dev(p.text());
                        headwords_in.insert(sanaadyanta_str.clone());
                        if !sanadi.is_empty() {
                            sanaadi_str = format!(
                                " + {} = {}",
                                sanadi
                                    .iter()
                                    .map(|x| x.name())
                                    .collect::<Vec<_>>()
                                    .join("+ "),
                                sanaadyanta_str
                            );
                        }
                    }

                    let entry_head = dev(&format!("{}{}", dhatu_str, sanaadi_str));
                    let mut definitions_d =
                        make_entry(self, entry_head, headwords_in, sanaadyanta, prayoga);
                    definitions.append(&mut definitions_d);

                    progress_bar.inc(1);
                }

                progress_bar.finish();
                info!("Got {} definitions.", definitions.len());

                let dest_file_path = dest_dir
                    .join(&dict_name)
                    .join(format!("{}.babylon", dict_name));
                self.dump_babylon(&dest_file_path, &definitions);
            }
        }
    }

    fn dump_babylon(&self, dest_path: &Path, definitions: &[Definition]) {
        if let Some(parent) = dest_path.parent() {
            fs::create_dir_all(parent).unwrap();
        }

        let mut content = String::new();
        for def in definitions {
            content.push_str(&def.headwords.join("|"));
            content.push_str("\n");
            content.push_str(&def.meaning);
            content.push_str("\n\n");
        }

        fs::write(dest_path, content).unwrap();
    }
}



fn main() {
    env_logger::init();

    let dict = BabylonDictionary::new(
        Path::new("/home/vvasuki/gitland/ambuda-org/vidyut-latest/kosha"),
    );

    // Uncomment the functions you want to run
    dict.dump_sanaadi_dicts(
        Path::new("/home/vvasuki/gitland/indic-dict/dicts/stardict-sanskrit-vyAkaraNa/kRdanta/vidyut/"),
        &SANAADI_DICT_KRDANTA,
        BabylonDictionary::get_krdanta_entry
    );
}
