REVEAL_META = {
    "title": "Rust im Linux Kernel",
    "author": "Alexander Böhm",
    "description": "Der Linux-Kernel ist in C und Assembler geschrieben. Linus Torvalds blockte in der Vergangenheit die Aufnahme von C++ kategorisch ab. Doch in 2022 schafften es Beiträge in den offiziellen Hauptentwicklungszweig, die die Unterstützung für die Entwicklung einer anderen Programmiersprache ermöglichen: das bei Mozilla 2006 entstandene Rust.  Rust verspricht bessere Ausdrucksmöglichkeiten und eine drastische Reduktion von bestimmten Sicherheitslücken bei ähnlicher Leistung wie C. Allerdings benötigt der Rust Compiler zusätzliche Abhängigkeiten (LLVM) und diverse Vorarbeiten (Abstractions, Bindings, Speichermanagement), um in allen Bereichen des Kernels nutzbar zu sein. Der Vortrag gibt einen historischen Abriss, geht auf Projekte und deren Herausforderungen in verschiedenen Dimensionen ein, stellt aktuelle Möglichkeiten in der Kernel-Entwicklung mit Rust dar und gibt einen Ausblick, was demnächst möglich sein soll.",
}

REVEAL_SLIDE_SEPARATOR = "---"
REVEAL_VERTICAL_SLIDE_SEPARATOR = "---~"

REVEAL_THEME = "white"

REVEAL_CONFIG = {
    "controls": False,
    "progress": True,
    "history": True,
    "slideNumber": True,
    "help": False,
    "transition": "none",  # none/fade/slide/convex/concave/zoom
    "viewDistance": 1,
    "mobileViewDistance": 0,
}
