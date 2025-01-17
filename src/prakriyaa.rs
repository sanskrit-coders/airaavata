use crate::util::{dev, slp};
use std::path::Path;
use std::sync::Arc;
use log::{error, info};
use vidyut_kosha::entries::{PadaEntry, SubantaEntry};
use vidyut_kosha::Kosha;
use vidyut_prakriya::args::{Dhatu, DhatuPada, Gana, Krdanta, Krt, Lakara, Muladhatu, Pratipadika, Prayoga, Purusha, Sanadi, Slp1String, Subanta, Tinanta, Vacana};
use vidyut_prakriya::args::BaseKrt::kta;
use vidyut_prakriya::{Dhatupatha, Prakriya, Vyakarana};

pub(crate) struct PrakriyaHelper {
    pub(crate) v: Arc<Vyakarana>,
    pub(crate) kosha: Arc<Kosha>,
    dhAtupATha: Dhatupatha,
}

impl PrakriyaHelper {
    pub(crate) fn new(data_path: &Path) -> Self {
        let v = Arc::new(Vyakarana::new());
        let kosha = Arc::new(Kosha::new(data_path.join("kosha/")).unwrap());
        let dhAtupATha = match Dhatupatha::from_path(data_path.join("data/dhatupatha.tsv")) {
            Ok(res) => res,
            Err(err) => {
                println!("{}", err);
                std::process::exit(1);
            }
        };
        let sUtrapATha = match Dhatupatha::from_path(data_path.join("data/sutrapatha.tsv")) {
            Ok(res) => res,
            Err(err) => {
                println!("{}", err);
                std::process::exit(1);
            }
        };
        Self { v, kosha, dhAtupATha}
    }

    fn show_prakriya(&self, prakriyas: Vec<Prakriya>) {
        for p in prakriyas {
            let mut steps = Vec::new();
            for step in p.history() {
                let sutra_text = self
                    .data
                    .get_sutra(&step.source, &step.rule().code())
                    .map(|s| dev(&s.text))
                    .unwrap_or_else(|| "(??)".to_string());

                let url = format!(
                    "[A](https://ashtadhyayi.github.io/suutra/{}/{}/)",
                    &step.rule().code()[..3],
                    step.rule().code()
                );
                let joined_result = step.result().iter().map(|x| x.text()).collect::<Vec<&str>>().join(",");
                let detail = format!(
                    "{} {} → {} {} {}",
                    dev(&step.rule().code()), // TODO: get type
                    step.rule().code(),
                    dev(joined_result),
                    sutra_text,
                    url
                );
                steps.push(detail);
            }

            info!("## {}\n{}\n", dev(p.text()), steps.join("  \n"));
        }
        
    }
    
    fn look_up_and_derive(&self, shabda: impl Into<String>) {
        let shabda = shabda.into();
        let entries = if shabda.chars().next().map_or(false, |c| c.is_ascii()) {
            self.kosha.get_all(&shabda)
        } else {
            self.kosha.get_all(&slp(&shabda))
        };

        if entries.is_empty() {
            error!("Can't get entry for {}", shabda);
            return;
        }

        for entry in entries {
            let prakriyas = match entry {
                PadaEntry::Subanta(s) => self.v.derive_subantas(&Subanta::builder().pratipadika(s.pratipadika_entry()).vacana(s.vacana()).linga(s.linga()).vibhakti(s.vibhakti()).build().unwrap()),
                _ => panic!("Expected BasicPratipadika")
            };

        }
    }

    fn derive_and_print_prakriya(&self) {
        let pada = Tinanta::builder().dhatu(Dhatu::mula(Slp1String::from("BU").unwrap(), Gana::Bhvadi)).prayoga(Prayoga::Kartari).pada( DhatuPada::Parasmaipada).lakara(Lakara::Lat).purusha(Purusha::Prathama).vacana(Vacana::Eka).build();

        // let spastaya = Dhatu::nama(Pratipadika::Basic(&slp("स्पष्ट")), Some(Sanadi::Ric));
        // let pada = Krdanta::builder().dhatu(spastaya).krt(Krt::Base(kta)).build();
 
        // match pada  {
        //     Tinanta(t) => self.print_prakriya(t),
        //     Krdanta(t) => self.print_prakriya(t),
        // }
    }
}
