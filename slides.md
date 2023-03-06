## Rust im Linux Kernel

<img src="media/Rust_for_Linux_logo.svg" alt="Logo Rust For Linux" width="30%"/>

Alexander Böhm

Chemnitzer Linux Tage, 11.03.2023

---

## Zu mir

* Kein Kernel-Entwickler \
  kleine Anpassungen, Spielereien, Experimente
* Früher hauptsächlich Python, Java, C/C++
* 2017 zaghafte Versuche mit Rust
* Seit 1.5y täglich Rust

---

## Sprachverteilung 

<img src="media/github-torvalds-linux-statistic.png" alt="C 98.5%, Assembly 0.8%" />


<small>

*Quelle: https://github.com/torvalds/linux, (4.3.23)*

</small>

---

### Neben C & Assembler

* Vereinzelte Projekte mit C++ und Ada
* Out Of Tree Entwicklungen
* 2006 Diskussion über C++: \
  Absage von diversen Kernel Maintainern

---

### Gründe gegen C++

* Komplexität Objekt-Orientierung
* Behandlung Sprachfunktionen? \
  Exceptions, Constructor, ...
* Unzureichende Kompilerunterstützung
* Strittige Kompatiblität mit C
* Kernel-Infrastruktur

---

## Sicherheit

<img src="media/mitre-cve-list-linux-kernel.png" alt="Liste von CVEs" />

<small>

*Quelle: cve.mitre.org, linux kernel (4.3.23)*

</small>

---

## Rust

---

### Geschichte

* Entstanden bei Mozilla 2009: \
  C/C++ zwar schnell, aber fehleranfällig
* Anforderung: \
  Einfache & sicher **Parallelisierung**
* LLVM-Kompiler
* 1. stabiles Release 2015
* Entwicklung hin zu systemnahen Use Cases

---

### Eigenschaften

* Strenge Typisierung
* Hohe Speichersicherheit
* Keine Garbage Collection
* Zero-Cost Abstraction
* Vergleichbare C/C++-Leistung
* Anbindung zu C

---

### Konzepte

* Traits, Generics, Optionals, Results
* Ownership & Lifetime Checks
* Macros
* Variable sind per default nicht veränderbar
* Panic Handler
* `unsafe` Code (C Interoperabilität)

---

### Kernel Developer

---

#### Rust for Linux

Miguel Ojeda, Wedson Almeida Filho, Alex Gaynor:

> We believe Rust offers key improvements over C in this domain.

---

#### Asahi Linux

Asahi Lina (M1 DRM Developer):

> Rust is truly magical! [...] It really guides you towards not just safe but good design.

---

## Verbesserungen

---

### Stack-based Pointer Leakage

---

#### C

```c
int* return_freed_stack() {
    // Wert wird auf Stack alloziiert
    int value = 42; 
    // Pointer zu value auf Stack
    return &value;
    // Stack wird abgeräumt
} // Rückgabe Pointer zeigt auf abgeräumten Stack 💥
```

Führt zu Warnung im GCC

```text
warning: function returns address of local variable 
         [-Wreturn-local-addr]
```

---

#### Rust

```rust
fn return_freed_stack() -> &i32 {
    // Variable auf dem Stack
    let value: i32 = 42;
    // Gebe Referenz auf Variable auf den Stack
    &value
    // Stack wird abgeräumt
}
```

Kompilierung scheitert:

```rust
error[E0106]: missing lifetime specifier
 --> stack-test.rs:1:24
  |
1 | fn smash_my_stack() -> &i32 {
  |                        ^ expected named lifetime parameter
  |
  = help: this function's return type contains a borrowed 
    value, but there is no value for it to be borrowed from
```

---

### Stack-based Buffer Overflow

---

#### C

```c
int my_stack_smash() {
    // Puffer auf dem Stack
    char buf[8] = { 0 };
    // Funktion, die den Puffer manipuliert 💣
    read_something(buf);
    return strncmp(buf, "true", sizeof(buf));
}

void read_something(char* value) {
    // Zu großer Puffer
    char* ups[256];
    // Überschreibe Stack-Grenzen 💥
    memcpy(value, ups, sizeof(ups));
}
```

Kompilierung ohne Probleme \
→ Programmabsturz mit `segfault`

---

#### Rust

```rust
use core::str::from_utf8;

fn my_stack_smash() -> i32 {
    let mut buf = [0u8; 8];
    read_something(&mut buf);
    if from_utf8(&buf).unwrap() == "true" { 1 } else { 0 }
}

fn read_something(value: &mut [u8]) {
    let buf = [0u8; 256];
    // Überschreibe Stack-Grenzen 💥
    value.copy_from_slice(&buf);
}
```

Kompilierung ohne Probleme \
→ Programmabsturz mit Panic Handler

```
thread 'main' panicked at 'source slice length (256) does
not match destination slice length (8)', src/main.rs:11:11
```

---

### Use after free

---

#### C

```c
// Rufe Funktion `do_something(-1)`
void* do_something(int value) {
    // Alloziiere Puffer
    void* buf = malloc(1024);
    if (value < 0) {
        free(buf);
        // Bug: return NULL vergessen
    }
    // Schreibe in Puffer
    memcpy(buf, &value, sizeof(value));
    // Gebe Pointer auf freien Speicher zurück
    return buf;
}
```

*GCC kompiliert ohne Fehler/Warnungen*

---

#### Rust

```rust
fn do_something(value: i32) -> Vec<u8> {
    // Alloziiere Puffer
    let mut buf = Vec::with_capacity(1024);
    if value < 0 {
        // Gebe explizit Speicher frei
        drop(buf);
    }
    // Schreibe in Puffer
    buf.copy_from_slice(&value.to_be_bytes());
    // Gebe Pointer auf freien Speicher zurück
    return buf;
}
```

---

*Kompilerfehler*

```rust
error[E0382]: borrow of moved value: `buf`
 --> src/main.rs:6:5
2 |     let mut buf = Vec::with_capacity(1024);
  |         ------- move occurs because `buf` has type ...
3 |     if value < 0 {
4 |         drop(buf);
  |              --- value moved here
5 |     }
6 |     buf.copy_from_slice(&value.to_be_bytes());
  |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
  |     value borrowed here after move
```

---

*Lösung die kompiliert wird*

```rust
fn do_something(value: i32) -> Option<Vec<u8>> {
    let mut buf = Vec::with_capacity(1024);
    if value < 0 {
        buf.copy_from_slice(&value.to_be_bytes());
        Some(buf)
    } else {
        None
    }
}
```

---

## Weg in den Kernel

---

### Die Anfänge

* 2012 LLVM Linux und Clang-Support
* 2020 Linux Plumbers Conference: \
  Vorschlag zum In-Tree Support von Rust

---

### Die Idee

![Rust Module Design](media/rust-module-design.png)

<small>

*Quelle: [Rust for Linux, Miguel Ojeda, Wedson Almeida Filho (März 2022)](https://www.youtube.com/watch?v=fVEeqo40IyQ&list=PL85XCvVPmGQgL3lqQD5ivLNLfdAdxbE_u)*

</small>

---

### 6.1 LTS Kernel

* Mitte Dezember 2022
* Minimales Kernelmodul
* Infrastruktur

---

![Rust Crate Infrastruktur](media/rust-infrastructure.png)

<small>

*Quelle: [Rust for Linux, Miguel Ojeda, Wedson Almeida Filho (März 2022)](https://www.youtube.com/watch?v=fVEeqo40IyQ&list=PL85XCvVPmGQgL3lqQD5ivLNLfdAdxbE_u)*

</small>

---

### 6.2er Kernel

* 20. Feburar 2023
* String-Behandlung
* Formater
* VTables-Unterstützung

---

### Aktuelle Entwicklungen

* netfilter
* Moduleparameter
* Dateisystemanbindung
* TCP-Server
* Einfache Treiber (Char/Misc Device)
* Arc-Datentyp (Asynchronous Resource Counter)
* Synchronisationsprimitive (Mutex, Semaphore)

---

## Erstes Kernelmodul

---

### Vorbereitung

* Rust Abhängigkeiten

```sh
rustup override set 1.66.0
rustup component add rust-src
cargo install --locked --version 0.56.0 bindgen
```

* LLVM/Clang Abhängigkeiten

```
apt-get install -y clang-11 lld-11 llvm-11
```

---

### ⚠️  Starke Versionsbhängigkeit ⚠️ 

* Meist abgestimmt auf konkrete Rust-Version

```text
*** Rust compiler 'rustc' is too new.
    This may or may not work.
***   Your version:     1.67.1
***   Expected version: 1.62.0
```

* Führt teilweise zu Fehlern

```text
error: the feature `core_ffi_c` has been stable since 1.64.0
       and no longer requires an attribute to enable
```

---

### Rust Support aktivieren

![General Setup -> Rust Support](media/linux-config-enable-rust.png)

---

### Beispiele aktivieren

![Kernel hacking -> Sample kernel code -> Rust samples](media/linux-config-enable-rust-sample.png)

---

### Module definieren

```rust
use kernel::prelude::*;

module! {
    type: RustCltModule,
    name: "rust_clt_module",
    author: "Rust for Linux Contributors",
    description: "Rust Module for CLT 2023",
    license: "GPL v2",
}
```

---

### Implementierung

```rust
struct RustCltModule;

impl kernel::Module for RustCltModule {
    fn init(
        name: &'static CStr,
        _module: &'static ThisModule
    ) -> Result<Self> {
        pr_info!("Hello from kernel module {name}!");
        Ok(Self {})
    }
}
```

---

### Kernel bauen

```
make LLVM=1 bzImage modules
```

---

## Aussicht

* Keine Kernel-Reimplementierung
* Abstimmungen mit Rust-Kompiler
* Offene Fragen bzgl. Distribution \
  (Versionsabhängigkeiten)

---

## Projekte

* Android IPC Binder
* [GPU Treiber für M1 (Asahi Linux)](https://asahilinux.org)
* [NVM Express Treiber](https://github.com/metaspace/linux/tree/nvme)
* [9p Server](https://github.com/wedsonaf/linux/commits/9p)

---

## Quellen/Referenzen

* [Rust For Linux](https://github.com/Rust-for-Linux/linux/tree/rust/Documentation/rust)
* [LKML: Vorschlag für Unterstützung von "in-tree" Rust Support](https://lore.kernel.org/lkml/CAKwvOdmuYc8rW_H4aQG4DsJzho=F+djd68fp7mzmBp3-wY--Uw@mail.gmail.com/T/#u)
* [Google Security Blog: Memory Safe Languages in Android 13](https://security.googleblog.com/2022/12/memory-safe-languages-in-android-13.html)
* [Linus Torvalds über C++ Pushbacks](http://www.uwsg.indiana.edu/hypermail/linux/kernel/0604.3/0964.html)
* [Stackoverflow Developer Survey 2022](https://survey.stackoverflow.co/2022/)
* [LWN: A first look at Rust in the 6.1 kernel](https://lwn.net/Articles/910762/)
* [LWN: A pair of Rust kernel modules](https://lwn.net/Articles/907685/)
* [Asahi Linux: Tales of the M1 GPU](https://asahilinux.org/2022/11/tales-of-the-m1-gpu/)
* [How Rust supports the Linux Kernel](https://www.youtube.com/watch?v=1R6CxuUwA7E)
* [Rust for Linux by Miguel Ojeda and Wedson Almeida Filho - Rust Linz, March 2022](https://www.youtube.com/watch?v=fVEeqo40IyQ)
* [Rust for Linux: Status and Wishlist](https://www.youtube.com/watch?v=fVEeqo40IyQ&list=PL85XCvVPmGQgL3lqQD5ivLNLfdAdxbE_u)
* [Rust for Linux, Rust CTCFT 2021](https://rust-lang.github.io/ctcft/slides/2021-11-22_-_Rust_CTCFT_-_Rust_for_Linux.pdf)
