# ADR 002: Wybór stosu technologicznego

## Kontekst
Potrzebujemy backendu o wysokiej wydajności, pełnej kontroli nad infrastrukturą i minimalnej liczbie zewnętrznych zależności SaaS.

## Decyzja
Serwis HTTP budujemy w Axum (Rust), zadania CPU-heavy w workerach (prefer Rust, opcjonalnie Python), baza to PostgreSQL, storage Hetzner S3, płatności Stripe, e-mail docker-mail-server. Frontend powstanie później w Svelte + Tailwind.

## Uzasadnienie
- Rust + Axum = szybkie, statycznie typowane API bez GC i GIL.  
- PostgreSQL zapewnia ACID, JSONB i replikację.  
- Hetzner S3 + self-hosted mail dają niezależność od AWS/GCP.  
- Stripe minimalizuje złożoność implementacji płatności przy wysokiej niezawodności.

## Konsekwencje
+ Jednolity toolchain (`cargo fmt`, `cargo clippy`, `cargo test`, `sqlx`).  
+ Pełna kontrola nad danymi i infrastrukturą.  
– Większa krzywa uczenia dla nowych agentów (Rust vs Python).  
– Konieczność utrzymania własnego mail servera.

## Komponenty jako serwisy
- **API (Axum)** — rdzeń HTTP, orkiestruje logikę domenową.  
- **Frontend (Svelte/Tailwind)** — SPA/SSR, konsumuje REST/GraphQL.  
- **Pricing microservices** — osobne kontenery (FDM, SLS, CNC), liczą wyceny.  
- **Email microservice** — moduł integrujący z docker-mail-server (wysyłka powiadomień).  
- Każdy komponent działa jako osobna usługa, komunikacja idzie po HTTP/kolejce/eventach.
