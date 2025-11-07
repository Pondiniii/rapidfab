# ADR 003: Architektura webservera Axum

## Kontekst
Core systemu to API-first backend, który musi obsłużyć jednoczesne uploady plików, działać przewidywalnie dla agentów i współpracować z pipeline workerów.

## Decyzja
Budujemy pojedynczą binarkę Axum w katalogu `services/api/` z modułami domenowymi w `services/api/src/app/<domena>`, middleware’ami `tower`, metrykami `/metrics` i kolejką zadań dla operacji CPU-heavy (spawn_blocking lub Redis Streams). Serwis ma własny `Containerfile`, `Makefile`, `README.md`, `docs/INDEX.md` i `tests/`, tak jak pozostałe mikroserwisy. Worker działa w osobnym procesie/kontenerze.

## Uzasadnienie
- Axum + Tokio zapewniają wysoką przepustowość bez blokowania innych requestów.  
- Jasna struktura modułów ułatwia orientację agentom.  
- Oddzielenie ciężkich zadań eliminuje ryzyko „przyduszenia” API pojedynczym żądaniem.

## Wpływ na skalowanie i współbieżność
- Tokio uruchamia pulę wątków roboczych, a każdy request staje się niezależnym taskiem async. Dwa równoległe żądania wyceny są obsługiwane równolegle – żadne nie blokuje drugiego, dopóki handler nie wykona długiej operacji synchronicznej.  
- Sekcje CPU-heavy są delegowane do `tokio::task::spawn_blocking` lub workerów; główny serwer tylko zapisuje metadane i odkłada zlecenie do kolejki. Dzięki temu nawet kosztowna analiza pliku nie zatrzymuje puli requestów.  
- Możemy nakładać limity (`tower::limit::ConcurrencyLayer`, semafory) oraz timeouts. Jeśli liczba równoległych zadań przekroczy próg, otrzymamy 429/timeout zamiast globalnej blokady.  
- Skalowanie poziome: binarkę Axum (stateless) można powielić za load balancerem, współdzieląc Postgresa, Redis i S3. Nie ma sesji w pamięci, więc rollout jest prosty.

### Obsługa zadań CPU-heavy
1. **spawn_blocking (baseline)** – Handler Axum odkłada pracę do `tokio::task::spawn_blocking`. Tokio utrzymuje oddzielną pulę wątków do zadań blokujących, więc główne wątki async obsługują dalej I/O. Limitujemy liczbę równoległych kalkulacji przez semafor, aby nie zakorkować CPU.  
2. **Osobny worker proces** – API zapisuje metadane, odkłada `job` do kolejki (Redis Streams / NATS). Worker (Rust lub Python) odbiera zadanie i wykonuje ciężką logikę, aktualizując status w bazie. API natychmiast zwraca `202 Accepted`. Pozwala to skalować workerów niezależnie.  
3. **Zewnętrzny mikroserwis** – W sytuacji, gdy analiza wymaga specyficznego runtime (np. Python z bibliotekami CAD), Axum wysyła żądania do dedykowanego mikroserwisu HTTP/gRPC. Mikroserwis zwraca wynik async lub odkłada go w kolejce. Pozwala to izolować technologicznie różne środowiska.

## Konsekwencje
+ Skalowalny serwer z ograniczonym zestawem bibliotek.  
+ Możliwość pionowego i poziomego skalowania przez powielenie binarki.  
+ Trzymanie API w `services/api/` ujednolica workflow z innymi serwisami (ten sam wzorzec Makefile/tests/docs).  
– Wymaga utrzymania dodatkowej kolejki i procesów workerów.  
– Większa złożoność przy debugowaniu zadań asynchronicznych.
