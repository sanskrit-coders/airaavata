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


PRAKRIYA_DHATU = "/home/vvasuki/gitland/vishvAsa/sanskrit/content/vyAkaraNam/pANinIyam/dhAtu-prakriyA/prakriyAH"
PRAKRIYA_SUP = "/home/vvasuki/gitland/vishvAsa/sanskrit/content/vyAkaraNam/pANinIyam/prAtipadika-prakriyA/sup/prakriyA"

DATA_PATH = "/home/vvasuki/gitland/ambuda-org/vidyut/vidyut-data/data/build/vidyut-latest/"

v = Vyakarana()
# data = Data("/home/vvasuki/gitland/ambuda-org/vidyut-latest/prakriya")
prakriya_data = Data(os.path.join(DATA_PATH, "prakriya/data"))
code_to_sutra = {(s.source, s.code): s.text for s in prakriya_data.load_sutras()}
# kosha = Kosha("/home/vvasuki/gitland/ambuda-org/vidyut/vidyut-data/data/build/vidyut-latest/kosha")
kosha = Kosha(os.path.join(DATA_PATH, "kosha"))


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
  return prakriyaas


def get_prakriyaa_str(prakriyas):
  prakriyaas = {}
  for p in prakriyas:
    steps = []
    for step in p.history:
      source = dev(step.source).replace('अष्टाध्यायी', 'अ')
      url = ""
      sutra = dev(code_to_sutra.get((step.source, step.code), "(??)"))
      if source == "अ":
        url = f"[A](https://ashtadhyayi.github.io/suutra/{step.code[:3]}/{step.code})"
      result = dev(','.join([x.text for x in step.result]))
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
  lookup_and_derive(pada, out_file_path=os.path.join(PRAKRIYA_DHATU, "tiNantAni"))


def derive_and_print_subanta(ref_pada, linga, vibhakti, vachana, type=None):
  if type == "nyap":
    pada = Pada.Subanta(
      pratipadika=Pratipadika.nyap(slp(ref_pada)),
      linga=linga,
      vibhakti=vibhakti,
      vacana=vachana,
    )
    lookup_and_derive(pada, out_file_path=PRAKRIYA_SUP)
  elif type == "kRt":
    entries = [x for x in kosha.get(slp(ref_pada)) if isinstance(x, PadaEntry.Tinanta)]
    for entry in entries:
      dhaatu = entry.dhatu_entry.dhatu
      anga = Pratipadika.krdanta(dhaatu, Krt.kvip)
      pada = Pada.Subanta(
        pratipadika=anga,
        linga=linga,
        vibhakti=vibhakti,
        vacana=vachana,
      )
      lookup_and_derive(pada, out_file_path=PRAKRIYA_SUP)

  else:
    pada = Pada.Subanta(
      pratipadika=Pratipadika.basic(slp(ref_pada)),
      linga=linga,
      vibhakti=vibhakti,
      vacana=vachana,
    )
    lookup_and_derive(pada, out_file_path=PRAKRIYA_SUP)




def dump_subantas(dest_dir="/home/vvasuki/gitland/vishvAsa/sanskrit/content/vyAkaraNam/pANinIyam/prAtipadika-prakriyA/sup/prakriyA"):
  praatipadika_str = "सुमनस्"
  # pratipadika=Pratipadika.nyap(slp(praatipadika_str))
  pratipadika=Pratipadika.basic(slp(praatipadika_str))
  content = ""
  lingas = [Linga.Pum, Linga.Napumsaka, Linga.Stri]
  for linga in lingas:
    content = f"{content}\n\n## {praatipadika_str} {dev(str(linga))}"
    for vibhakti in Vibhakti.choices():
      for vacana in Vacana.choices():
        prakriyaas = lookup_and_derive(shabda=Pada.Subanta(
          pratipadika, linga, vibhakti, vacana
        ))
        for result, prakriya_str in prakriyaas.items():
          content += f"\n\n{prakriya_str.replace('##', '###')}"
  file_path = os.path.join(dest_dir, file_helper.get_storage_name(text=praatipadika_str) + ".md")
  os.makedirs(os.path.dirname(file_path), exist_ok=True)
  md_file = MdFile(file_path)
  md_file.dump_to_file(metadata={"title": praatipadika_str}, content=content, dry_run=False)


def dump_tinantas(ref_pada, dest_dir=os.path.join(PRAKRIYA_DHATU, "tiNantAni")):
  entries = [x for x in kosha.get(slp(ref_pada)) if isinstance(x, PadaEntry.Tinanta)]
  dhaatu = entries[0].dhatu_entry.dhatu
  dhaatu_str = dev(dhaatu.aupadeshika)
  # dhaatu_str = "दाञ्"
  # dhaatu = Dhatu.mula(aupadeshika=slp(dhaatu_str), gana=Gana.Juhotyadi)
  content = ""
  prayogas = [Prayoga.Kartari, Prayoga.Karmani]
  for prayoga in prayogas:
    for lakara in Lakara.choices():
      content = f"{content}\n\n## {dhaatu_str} {dev(str(lakara))} {dev(str(prayoga))}"
      for parasmai_mode in [DhatuPada.Parasmaipada, DhatuPada.Atmanepada]:
        pada = Pada.Tinanta(
          dhatu=dhaatu,
          prayoga=prayoga,
          dhatu_pada=parasmai_mode,
          lakara=lakara,
          purusha=Purusha.Prathama,
          vacana=Vacana.Bahu,
        )
        prakriyaas = lookup_and_derive(pada)
        if len(prakriyaas) == 0:
          continue
        if prayoga != Prayoga.Karmani:
          content = f"{content}\n\n### {dev(str(parasmai_mode))}"
        for purusha in Purusha.choices():
          content = f"{content}\n\n#### {dev(str(purusha))}"
          for vacana in Vacana.choices():
            content = f"{content}\n\n##### {dev(str(vacana))}"
            pada = Pada.Tinanta(
              dhatu=dhaatu,
              prayoga=prayoga,
              dhatu_pada=parasmai_mode,
              lakara=lakara,
              purusha=purusha,
              vacana=vacana,
            )
            prakriyaas = lookup_and_derive(pada)
            for result, prakriya_str in prakriyaas.items():
              content += f"\n\n{prakriya_str.replace('##', '######')}"
  title = f"{dhaatu_str} {dev(dhaatu.gana)}"
  file_path = os.path.join(dest_dir, file_helper.get_storage_name(text=title) + ".md")
  os.makedirs(os.path.dirname(file_path), exist_ok=True)
  md_file = MdFile(file_path)
  md_file.dump_to_file(metadata={"title": title}, content=content, dry_run=False)


def dump_kRdantas(ref_pada, dest_dir=os.path.join(PRAKRIYA_DHATU, "kRdantAni")):
  entries = [x for x in kosha.get(slp(ref_pada)) if isinstance(x, PadaEntry.Tinanta)]
  for entry in entries:
    dhaatu = entry.dhatu_entry.dhatu
    dhaatu_str = dev(dhaatu.aupadeshika)
    # dhaatu_str = "डुदाञ्"
    # dhaatu = Dhatu.mula(aupadeshika=slp(dhaatu_str), gana=Gana.Juhotyadi)
    content = ""
    for kRt in Krt.choices():
      anga = Pratipadika.krdanta(dhaatu, kRt)
      prakriyaas = lookup_and_derive(anga)
      if len(prakriyaas) == 0:
        continue
      content = f"{content}\n\n## {dhaatu_str} + {dev(str(kRt))}"
      for result, prakriya_str in prakriyaas.items():
        content += f"\n\n{prakriya_str.replace('##', '###')}"
    title = f"{dhaatu_str} {dev(dhaatu.gana)}"
    file_path = os.path.join(dest_dir, file_helper.get_storage_name(text=title) + ".md")
    os.makedirs(os.path.dirname(file_path), exist_ok=True)
    md_file = MdFile(file_path)
    md_file.dump_to_file(metadata={"title": title}, content=content, dry_run=False)


def derive_and_print_kRdanta():
  spastaya = Dhatu.nama(Pratipadika.basic(slp("स्पष्ट")), nama_sanadi=Sanadi.Ric)
  kRdanta = Pratipadika.krdanta(spastaya, krt=Krt.kta)
  lookup_and_derive(kRdanta)


if __name__ == '__main__':
  derive_and_print_subanta(ref_pada="श्रयति", type="kRt", linga=Linga.Stri, vibhakti=Vibhakti.Sasthi, vachana=Vacana.Eka)
  # derive_and_print_tinanta()
  # lookup_and_derive("पद्ये", out_file_path=os.path.join(PRAKRIYA_DHATU, "tiNantAni"), type=PadaEntry.Tinanta)
  # dump_subantas()
  # dump_tinantas(ref_pada="पद्ये")
  # dump_kRdantas(ref_pada="बृंहिता")
  pass
  