import logging
import os
from copy import copy

import indic_transliteration
from curation_utils import file_helper
from doc_curation.md.file import MdFile
from indic_transliteration.vidyut_helper import dev, slp

import regex
from click.utils import make_default_short_help
from sanskrit_data.collection_helper import OrderedSet
from tqdm import tqdm
from vidyut.kosha import Kosha, PratipadikaEntry, PadaEntry
from vidyut.lipi import transliterate, Scheme
from vidyut.prakriya import Data, Vyakarana, Sanadi, Krt, Pratipadika, Vibhakti, Vacana, Linga, Pada, Taddhita, \
  DhatuPada, Lakara, Purusha, Prayoga, Gana, Dhatu

# Remove all handlers associated with the root logger object.
for handler in logging.root.handlers[:]:
  logging.root.removeHandler(handler)
logging.basicConfig(
  level=logging.DEBUG,
  format="%(levelname)s:%(asctime)s:%(module)s:%(lineno)d %(message)s")


PRAKRIYA_BASE = "/home/vvasuki/gitland/vishvAsa/sanskrit/content/vyAkaraNam/pANinIyam/prakriyAH"


v = Vyakarana()
data = Data("/home/vvasuki/gitland/ambuda-org/vidyut-latest/prakriya")
code_to_sutra = {(s.source, s.code): s.text for s in data.load_sutras()}
kosha = Kosha("/home/vvasuki/gitland/ambuda-org/vidyut-latest/kosha")


def lookup_and_derive(shabda, type=None, out_file_path=None):
  if isinstance(shabda, str):
    entries = kosha.get(slp(shabda))
  else:
    entries = [shabda]
  if type is not None:
    entries = [x for x in entries if isinstance(x, type)]
  if len(entries) == 0:
    logging.error(f"Can't get entry for {shabda}.")
    return
  for entry in entries:
    prakriyas = v.derive(entry)
    prakriyaas = get_prakriyaa_str(prakriyas)
    for result, prakriya_str in prakriyaas.items(): 
      if out_file_path is not None:
        file_path = os.path.join(out_file_path, file_helper.get_storage_name(text=result) + ".md")
        os.makedirs(os.path.dirname(file_path), exist_ok=True)
        md_file = MdFile(file_path)
        md_file.dump_to_file(metadata={"title": result}, content=prakriya_str, dry_run=False)
  pass


def get_prakriyaa_str(prakriyas):
  prakriyaas = {}
  for p in prakriyas:
    steps = []
    for step in p.history:
      source = dev(step.source).replace('आस्ह्तद्ह्ययि', 'अष्टाध्यायी')
      url = ""
      if source == "अष्टाध्यायी":
        sutra = dev(code_to_sutra.get((step.source, step.code), "(??)"))
        url = f"[A](https://ashtadhyayi.github.io/suutra/{step.code[:3]}/{step.code})"
      result = dev(','.join(step.result))
      detail = f"{source} {step.code} → {result} ({sutra} {url})"
      steps.append(detail)
    md_newline = '  \n'
    result = dev(p.text)
    prakriyaa_str = f"## {result}\n{md_newline.join(steps)}\n"
    prakriyaas[result] = prakriyaa_str
  
  return prakriyaas


def derive_and_print_tinanta():
  pada = Pada.Tinanta(
    dhatu=Dhatu.mula(aupadeshika="BU", gana=Gana.Bhvadi),
    prayoga=Prayoga.Kartari,
    lakara=Lakara.Lat,
    purusha=Purusha.Prathama,
    vacana=Vacana.Eka,
  )
  lookup_and_derive(pada, out_file_path=os.path.join(PRAKRIYA_BASE, "tiNantAni"))


def derive_and_print_subanta():
  # pada = Pada.Subanta(
  #   pratipadika=Pratipadika.basic(slp("सुमनस्")),
  #   linga=Linga.Pum,
  #   vibhakti=Vibhakti.Prathama,
  #   vacana=Vacana.Eka,
  # )
  pada = Pada.Subanta(
    pratipadika=Pratipadika.basic(slp("नदी")),
    linga=Linga.Stri,
    vibhakti=Vibhakti.Prathama,
    vacana=Vacana.Eka,
  )
  lookup_and_derive(pada, out_file_path=os.path.join(PRAKRIYA_BASE, "subantAni"))


def derive_and_print_kRdanta():
  spastaya = Dhatu.nama(Pratipadika.basic(slp("स्पष्ट")), nama_sanadi=Sanadi.Ric)
  kRdanta = Pratipadika.krdanta(spastaya, krt=Krt.kta)
  lookup_and_derive(kRdanta)


if __name__ == '__main__':
  # derive_and_print_subanta()
  # derive_and_print_tinanta()
  lookup_and_derive("चोरयति", out_file_path=os.path.join(PRAKRIYA_BASE, "tiNantAni"), type=PadaEntry.Tinanta)
  pass
  