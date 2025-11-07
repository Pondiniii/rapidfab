# ADR 005: Mikroserwisy do modułów wyceny

## Kontekst
Moduł wyceny może różnić się w zależności od technologii (FDM, SLS, CNC itd.). Rozszerzanie monolitu o kolejne algorytmy zwiększa złożoność i utrudnia skalowanie zespołu agentów. Chcemy traktować wyceny jak „klocki”, które można dodawać niezależnie.

## Decyzja
Rozdzielamy logikę wycen do osobnych mikroserwisów (np. `pricing-fdm`, `pricing-sls`). Core Axum pełni rolę koordynatora: przyjmuje żądanie, waliduje dane, deleguje obliczenia do właściwego mikroserwisu, a po otrzymaniu wyniku aktualizuje bazę. Każdy mikroserwis działa w kontenerze rootless i jest wdrażany niezależnie.

## Uzasadnienie
- Umożliwia niezależny rozwój i deployment modułów wyceny przez różnych agentów.  
- Łatwo dodać nową technologię (nowy serwis) bez zmiany core API.  
- Mikroserwisy można pisać w innym języku, jeśli dana technologia wymaga specyficznych bibliotek (np. Python dla CAD).  
- Rootless container minimalizuje ryzyko bezpieczeństwa i upraszcza uruchamianie na tańszych VPS-ach.

## Architektura przepływu
1. Axum core zapisuje żądanie wyceny w Postgres. Na etapie MVP może od razu wywołać mikroserwis przez HTTP (`POST /pricing/<tech>/quote`) i poczekać na wynik.  
2. Docelowo Axum publikuje event (`quote.requested`) na kolejce (Redis Streams/NATS).  
3. Mikroserwis (np. `pricing-fdm`) nasłuchuje tylko na swoje typy zadań, wykonuje kalkulację i publikuje `quote.completed` (lub `quote.failed`).  
4. Axum core konsumuje aktualizację z kolejki, zapisuje wynik w bazie i udostępnia go przez API (np. `GET /quotes/{id}`).  
5. Wszystkie serwisy pakujemy jako rootless kontenery (`podman`/`docker --userns=keep-id`), co umożliwia bezpieczne uruchomienie wielu instancji na jednym VPS bez praw roota.

### Scenariusze żądań wyceny
1. **Nowy plik + wycena**  
   - Użytkownik uploaduje plik (Axum → storage).  
   - Axum zapisuje `file_id`, tworzy rekord wyceny i deleguje zadanie do mikroserwisu.  
2. **Wycena istniejącego pliku**  
   - Użytkownik wybiera wcześniejszy `file_id`, modyfikuje parametry (materiał, SLA).  
   - Axum tylko wysyła referencję do pliku (bez ponownego uploadu) i otrzymuje nowy wynik.  
3. **Rewizja wyceny**  
   - Mikroserwis może otrzymać `quote_id` i policzyć ponownie (np. po zmianie algorytmu).  
   - Wynik zapisywany jako nowa wersja lub aktualizacja w historii.

### Dlaczego kolejka (Redis/NATS) jest docelowym wyborem
- Oddziela lifecycle requestu HTTP od ciężkiej analizy – API zwraca 202 natychmiast, wynik dostarczamy asynchronicznie.  
- Zapewnia spójny mechanizm retry i odporność na awarie (zadanie zostaje w kolejce, dopóki worker go nie potwierdzi).  
- Umożliwia skalowanie workerów bez dotykania API i utrzymuje jednolity kontrakt eventów dla różnych technologii (FDM, SLS, CNC).  
- Nadal zachowujemy prosty start: na etapie MVP można użyć bezpośredniego HTTP i w dowolnej chwili przełączyć się na kolejkę.

### Technologia mikroserwisów
- Mikroserwisy pricingowe domyślnie piszemy w Pythonie (FastAPI/Flask + pydantic), bo biblioteki CAD/mesh są łatwiej dostępne i development jest szybszy.  
- W przyszłości można przepisać konkretny serwis na Rust, jeśli wymagane są dodatkowe performance/security features.  
- Każdy mikroserwis udostępnia API HTTP (baseline) oraz konsumenta eventów (docelowo), co pozwala płynnie migrować z wariantu synchronicznego na asynchroniczny.

### Kontrakty i testy
- Każdy mikroserwis ma jawny kontrakt (OpenAPI + schema eventów).  
- End-to-end tests (Axum ↔ pricing) oraz unit dla algorytmów wycen są obowiązkowe – coding-agent pisze testy, test-agent waliduje.  
- Mocki eventów i HTTP endpointów publikujemy w repo, żeby agenci mogli lokalnie symulować interakcje.

## Konsekwencje
+ Moduły wyceny są izolowane technologicznie i operacyjnie.  
+ Można niezależnie skalować liczbę instancji dla FDM/SLS/CNC w zależności od obciążenia.  
+ Łatwe AB-testy i iteracje algorytmów (deploy nowej wersji konkretnego mikroserwisu).  
– Wymaga infrastruktury kolejki i mechanizmu discovery/konfiguracji endpointów.  
– Więcej serwisów oznacza dodatkowe monitorowanie i logowanie między komponentami.  
– Potrzeba kontraktu eventów (schematy), aby uniknąć rozjazdów między serwisami.

## Status
Proposed
