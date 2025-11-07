# ADR 006: Mikroserwis e-mail + docker-mail-server

## Kontekst
Projekt wymaga pełnej kontroli nad wysyłką maili (brak SaaS typu Mailgun). Chcemy korzystać z `docker-mail-server` (DMS) jako własnego serwera SMTP, ale jednocześnie potrzebujemy prostego, testowalnego API do generowania i wysyłania wiadomości z aplikacji. Serwis musi być minimalny, szybki i łatwy do wymiany.

## Decyzja
Tworzymy dedykowany mikroserwis `email-service`, który wystawia proste HTTP API (np. `POST /send`). Serwis renderuje template (Jinja/Handlebars), wysyła wiadomość przez SMTP do `docker-mail-server` i zwraca wynik. Na etapie MVP Axum wywołuje API synchronicznie; docelowo wprowadzamy kolejkę (Redis Streams/NATS) jako bufor i retry.

## Uzasadnienie
- Izolacja: logika templatingu, retry i logowania błędów maili jest w osobnym komponencie – core API pozostaje czyste.  
- Kontrola: własny SMTP (DMS) pozwala zarządzać reputacją, konfiguracją DKIM/DMARC bez zewnętrznych zależności.  
- Minimalizm: mały kontener, jedno API; łatwo go reemplikować lub zastąpić inną implementacją.  
- Skalowanie: w razie potrzeby zwiększamy liczbę instancji lub dodajemy kolejkę bez dotykania reszty systemu.

## Architektura
1. `docker-mail-server` działa jako samodzielny kontener z konfiguracją (DKIM, użytkownicy, aliasy) w volume.  
2. `email-service` (Python/FastAPI lub Rust/Axum) przyjmuje żądania z core API.  
   - **Synchronicznie (MVP)**: `POST /send` → render template → SMTP do DMS → zwrot statusu.  
   - **Docelowo**: Axum publikuje event `email.requested` do kolejki; `email-service` konsumuje, wysyła mail i publikuje `email.sent` / `email.failed`.  
3. Logi sukcesów/porażek trafiają do Postgresa (`email_events`) oraz do logów centralnych.  
4. Template’y i konfiguracja (adresy nadawców) przechowywane w repo `email-service` lub w S3 z wersjonowaniem.

## Kontrakty i testy
- HTTP API opisane w OpenAPI; payload zawiera template_id, język, kontekst (JSON), adresata.  
- Testy unit (rendering, walidacja) + integracyjne (SMTP stub, np. `mailhog`).  
- Mocki API/email-eventów publikujemy w repo, żeby Axum mógł symulować wysyłkę bez realnego SMTP.

## Konsekwencje
+ Łatwo wymienić lub skalować wysyłkę maili bez dotykania core API.  
+ Możliwość dodania dodatkowych workerów/retry (kolejka).  
– Dodatkowy serwis wymaga monitoringu i konfiguracji (SMTP, certyfikaty).  
– Trzeba zadbać o reputację i konfigurację DMS (SPF, DKIM, DMARC).

## Status
Proposed
