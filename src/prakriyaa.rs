use std::path::Path;
use std::sync::Arc;
use vidyut_kosha::Kosha;
use vidyut_prakriya::args::{Dhatu, DhatuPada, Gana, Lakara, Prayoga, Purusha, Tinanta, Vacana};
use vidyut_prakriya::Vyakarana;

struct PrakriyaHelper {
    v: Arc<Vyakarana>,
    kosha: Arc<Kosha>,
    data: Arc<Data>,
}

impl PrakriyaHelper {
    fn new(data_path: &Path, kosha_path: &Path) -> Self {
        let data = Arc::new(Data::new(data_path));
        let kosha = Arc::new(Kosha::new(kosha_path));
        let v = Arc::new(Vyakarana::new());

        Self { v, kosha, data }
    }

    fn print_prakriya(&self, shabda: impl Into<String>) {
        let shabda = shabda.into();
        let entries = if shabda.chars().next().map_or(false, |c| c.is_ascii()) {
            self.kosha.get(&shabda)
        } else {
            self.kosha.get(&self.slp(&shabda))
        };

        if entries.is_empty() {
            error!("Can't get entry for {}", shabda);
            return;
        }

        for entry in entries {
            let prakriyas = self.v.derive(entry);
            for p in prakriyas {
                let mut steps = Vec::new();
                for step in p.history() {
                    let sutra_text = self
                        .data
                        .get_sutra(&step.source, &step.code)
                        .map(|s| self.dev(&s.text))
                        .unwrap_or_else(|| "(??)".to_string());

                    let url = format!(
                        "[A](https://ashtadhyayi.github.io/suutra/{}/{}/)",
                        &step.code[..3],
                        step.code
                    );

                    let detail = format!(
                        "{} {} → {} {} {}",
                        self.dev(&step.source),
                        step.code,
                        self.dev(&step.result.join(",")),
                        sutra_text,
                        url
                    );
                    steps.push(detail);
                }

                info!("## {}\n{}\n", self.dev(p.text()), steps.join("  \n"));
            }
        }
    }

    fn derive_and_print_tinanta(&self) {
        let pada = Tinanta::builder() {
            dhatu: Dhatu::mula("BU", Gana::Bhvadi),
            prayoga: Prayoga::Kartari,
            lakara: Lakara::Lat,
            purusha: Purusha::Prathama,
            vacana: Vacana::Eka,
            pada: DhatuPada::Parasmaipada,
            skip_at_agama: false
        };
        self.print_prakriya(pada);
    }

    fn derive_and_print_krdanta(&self) {
        let spastaya = Dhatu::nama(Pratipadika::basic(&self.slp("स्पष्ट")), Some(Sanadi::Ric));
        let krdanta = Pratipadika::krdanta(spastaya, Krt::Kta);
        self.print_prakriya(krdanta);
    }
}
