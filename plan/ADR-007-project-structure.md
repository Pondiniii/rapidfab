# ADR 007: Struktura katalogów projektu

## Kontekst
RapidFab.xyz to zestaw konteneryzowanych serwisów (API w Axum, pricing FDM, e-mail, przyszłe Stripe itp.), dokumentacji i testów. Każdy agent LLM musi szybko odnaleźć kod, dokumentację roboczą i testy konkretnego komponentu, a także mieć jedną oczywistą warstwę uruchomieniową (Makefile + docker-compose). Dotychczasowe rozważania (`backend/` obok `services/`, warianty apps/core) wprowadzały asymetrię lub utrudniały ustawienie wspólnych narzędzi.

## Decyzja
Repozytorium organizujemy symetrycznie według katalogu `services/`, w którym **każdy** serwis (w tym API Axum) ma takie same artefakty. Struktura root jest minimalna:

```
.
├── Makefile                # globalne cele: build, test-e2e, compose
├── docker-compose.yml      # uruchamia całą platformę (API + pricing + e-mail + obserwowalność)
├── plan/                   # PRD/ADR
├── tests/                  # globalne testy e2e/kontraktowe/fixtures
│   ├── e2e/
│   ├── contracts/
│   └── fixtures/
├── services/
│   ├── api/                # Axum backend – również mikroserwis
│   │   ├── Containerfile
│   │   ├── Makefile        # lokalne cele: lint/fmt/test/build
│   │   ├── README.md       # szybki opis i instrukcje
│   │   ├── docs/
│   │   │   └── INDEX.md    # punkt startowy + miejsce na notatki (np. docs/work/*.md)
│   │   ├── tests/          # testy jednostkowe/integracyjne specyficzne dla serwisu
│   │   └── src/...         # kod właściwy
│   ├── pricing-fdm/
│   │   ├── Containerfile
│   │   ├── Makefile
│   │   ├── README.md
│   │   ├── docs/INDEX.md
│   │   ├── docs/work/...
│   │   └── tests/
│   ├── email-service/
│   │   ├── Containerfile
│   │   ├── Makefile
│   │   ├── README.md
│   │   ├── docs/INDEX.md
│   │   ├── docs/work/...
│   │   └── tests/
│   └── (kolejne serwisy analogicznie)
├── infra/                  # artefakty infra poza docker-compose (np. k8s/)
│   └── k8s/
└── docs/                   # (opcjonalnie) przekrojowa dokumentacja ogólna repo
```

Zasady:
- każdy serwis jest niezależnym kontenerem i ma własny `Containerfile`, `Makefile`, `README.md`, `docs/INDEX.md` oraz katalog `tests/`;
- w `docs/` serwisu agent może trzymać kontekst pracy (`docs/work/*.md`, diagramy itp.), zachowując `INDEX.md` jako spis treści;
- globalny `tests/` przechowuje scenariusze e2e, kontrakty i fixture’y współdzielone między serwisami;
- `Makefile` i `docker-compose.yml` w root wymuszają jednolity sposób uruchamiania i testowania całej platformy;
- katalog `infra/` zawiera tylko to, co nie mieści się w Compose (np. manifesty Kubernetes, Terraform); jeśli kiedyś nie będzie potrzebny, można go usunąć bez naruszania reszty struktury.

## Uzasadnienie
- Symetria (`services/<nazwa>`) eliminuje wyjątki – każdy komponent wygląda identycznie, co przyspiesza onboarding LLM-agenta.
- Wymuszone artefakty (Containerfile, Makefile, docs, tests) gwarantują, że serwis jest gotowy do konteneryzacji, ma lokalne workflow i miejsce na wiedzę domenową.
- Globalny `Makefile` + `docker-compose.yml` obsługują cały pipeline (build/test/e2e) i ułatwiają test-agentowi walidację pracy coding-agenta.
- Oddzielenie testów serwisowych (`services/*/tests`) od systemowych (`tests/`) klarownie dzieli odpowiedzialność (feature vs. e2e).
- Minimalny root skraca czas wyszukiwania – agent widzi tylko plan, testy globalne i katalog z usługami.

## Konsekwencje
+ Dodanie nowego serwisu = skopiowanie szablonu (`Containerfile`, `Makefile`, `README`, `docs/INDEX.md`, `tests/`) i wpisanie go do `docker-compose.yml`.  
+ Każdy serwis jest gotowy do publikacji jako kontener (rootless) i można nim zarządzać niezależnie.  
+ Dokumentacja robocza pozostaje blisko kodu, co zwiększa spójność z filozofią LLM Agent First.  
− Trzeba pilnować spójności lokalnych Makefile’i (konwencja targetów `fmt`, `lint`, `test`).  
− Więcej plików w każdym serwisie na start (szablony), co wymaga dyscypliny przy zakładaniu nowych katalogów.  
− `infra/` pozostaje dodatkowym katalogiem – gdy nieużywany, należy go czyścić, by zachować minimalizm.

## Status
Accepted
