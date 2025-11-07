# RapidFab.xyz — karta projektu (LLM Agent First)

## Po co istnieje ten dokument
Ujednolica kierunek pracy nad rapidfab.xyz i zapewnia, że każdy agent LLM rozumie wizję: minimalny, modularny hub wycen on-demand podobny do Xometry, lecz prostszy, tańszy i szybszy. Repozytorium ma być **LLM Agent First** – mała liczba plików, przewidywalna struktura katalogów, jasne kontrakty API i zasady współpracy między agentami.

## Cel produktu
- Stworzyć platformę „upload → instant quote → buy” dla druku 3D/CNC/MJF, działającą szybko i czytelnie na desktop/mobile.
- Backend ma być banalny do zrozumienia, modularny i bezpieczny, z łatwym wdrażaniem nowych funkcji bez kruszenia całości.
- System obsługuje wielu użytkowników (klienci, admin), jest stabilny i operuje w kosztach ~10 USD VPS + Hetzner S3.
- Zapewnić pełną kontrolę nad infrastrukturą (brak vendor lock-in typu AWS/GCP Mailgun).

## Filozofia i zasady LLM Agent First
- **Minimalizm katalogów i kodu**: jeden moduł = jeden cel, zero zbędnych warstw.
- **Jasne kontrakty**: każdy endpoint opisany w OpenAPI, kluczowe funkcje/trait-y mają krótkie doc-komentarze z efektem.
- **Deterministyczne flow**: brak ukrytej magii, brak globalnego stanu i side effectów poza warstwą infrastruktury.
- **Dokumentacja w repo**: krótkie pliki `.md` w katalogach głównych zamiast osobnych wiki.
- **Automatyzowalne zadania**: skrypty `make`/`invoke` z parametrami zrozumiałymi dla agentów.

## Styl kodowania
- **Less is more**: preferuj krótkie moduły Rust (< 300 linii); jeśli rośnie → rozbij na podmoduły.  
- **SOLID / DRY / KISS**: brak powtórzeń, proste nazewnictwo (`verb_subject`), jawne interfejsy traits.  
- **Czytelność > „spryt”**: unikamy makr, dopóki nie poprawiają czytelności; dependency injection przez konstruktor/trait.  
- **Konwencja**: `cargo fmt`, `cargo clippy -D warnings`, testy (`cargo test`). W Python workerach `ruff` + `pytest` jeśli są użyte.  
- **Logging i błędy**: `tracing` z poziomami INFO/WARN, błędy mapowane na struktury JSON, brak stack trace w odpowiedzi REST.  
