# 🎯 PresenterKit

**Lightweight, portable Windows presenter overlay tool for teachers and graphic software presentations.**

Navrhnuté špeciálne pre výučbu softvérov ako **Adobe Photoshop, Adobe Illustrator a Affinity Designer** bez konfliktov s ich klávesovými skratkami.

---

## ✨ Kľúčové vlastnosti

- 🚀 **100% Portable & Lightweight:** Žiadna inštalácia, žiadne EULA, žiadne zbytočné knižnice ani .NET runtime. Spustiteľné priamo z USB kľúča (~3–4 MB).
- 🔦 **Spotlight (bodové nasvietenie):** Stmaví celú obrazovku s jemným Apple-style gradientom a nechá osvetlený iba bezprostredný priestor okolo myši.
- 🔍 **Magnifier (lupa s chromatickou aberáciou):** Označte kruhovú oblasť a lupa sa vysunie vedľa nej s decentným tieňom, prepojená elegantnou šípkou. Podpora viacerých láp naraz.
- ❄️ **Region Freeze (parciálne zmrazenie obrazovky):** Označte obdĺžnik alebo polovicu obrazovky, ktorá ostane zamrznutá v čase, kým zvyšok obrazovky funguje normálne.
- ✦ **Mouse Comet Trail:** Hladká, prémiová stopa za kurzorom myši s dynamickou žiarou podľa rýchlosti pohybu.
- 🎛️ **System Tray:** Beží diskrétne na pozadí, minimálna spotreba RAM (< 35 MB) a CPU (< 0.1% v idle).

---

## ⌨️ Klávesové skratky

| Kláves | Akcia | Popis |
| :--- | :--- | :--- |
| **`ScrollLock`** *(podržanie)* | **Spotlight** | Stmaví obrazovku okolo kurzora, jemný prechod |
| **`Home`** *(podržanie + ťah + pustenie)* | **Magnifier (Lupa)** | Nakreslí kruh, po pustení vytvorí zväčšovacie sklo so šípkou |
| **`Insert`** *(podržanie + ťah + pustenie)* | **Freeze Region** | Zamrazí vybraný obdĺžnik na obrazovke |
| **`End`** *(stlačenie)* | **Freeze Half Screen** | Prepína a zamrazí ľavú / pravú polovicu obrazovky |
| **`Page Up`** / **`Page Down`** | **Zväčšenie / Zmenšenie** | Upraví veľkosť kruhu spotlightu |
| **`Delete`** *(stlačenie)* | **Zmazať posledný efekt** | Odstráni naposledy pridanú lupu alebo freeze |
| **`Escape`** *(stlačenie)* | **Vyčistiť všetko** | Odstráni všetky aktívne lupy a zmrazené regióny |

---

## 📥 Ako stiahnuť hotovú aplikáciu pre Windows

Aplikácia sa automaticky kompiluje cez **GitHub Actions**:

1. Prejdite na záložku **[Actions](../../actions)** v tomto GitHub repozitári.
2. Kliknite na najnovší úspešný beh workflowu (**Build & Release PresenterKit**).
3. V sekcii **Artifacts** na spodku stránky kliknite na **`presenter-kit-windows`**.
4. Stiahne sa zip archív obsahujúci:
   - `presenter-kit.exe`
   - `settings.toml`
5. Rozbaľte archív (napr. na USB kľúč) a dvojklikom spustite `presenter-kit.exe`. Hotovo!

---

## 🛠️ Lokálny build (pre vývojárov)

Ak máte nainštalovaný Rust na Windows:

```bash
cargo build --release
```

Výsledná binárka sa nachádza v:
`target/release/presenter-kit.exe`
