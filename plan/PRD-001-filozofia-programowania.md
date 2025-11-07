# PRD 001: Filozofia programowania RapidFab.xyz

## Kontekst
Tworzymy minimalistyczny marketplace produkcyjny, rozwijany przez agentów LLM.
Prostota i minimalizm projektu to core rules projektu.
tak aby agenci LLM mogli prosto programować.

## Wymagania
- Minimalna liczba plików i warstw.  
- Jasne kontrakty (OpenAPI, doc-komentarze) dla każdego modułu.  
- Ścisłe stosowanie zasad SOLID / DRY / KISS.  
- Struktura katalogów przewidywalna i powtarzalna dla agentów.  
- Serwisy izolowane w kontenerach (rootless), aby każdy komponent był wymienialny jak klocek.  
- Cała aplikacja budowana w filozofii mikroserwisów: każdy kontekst domenowy to niezależny serwis o jasnym API.  
- Day 0: wszystkie mikroserwisy budujemy i uruchamiamy w rootless kontenerach (Podman/Docker z userns), nawet lokalnie.

## Decyzja
Wprowadzamy limit ~300 linii na moduł, dzielimy odpowiedzialności na serwisy (logika) i repozytoria (dostęp do danych), każdą usługę pakujemy w osobny kontener (rootless), a wszystkie decyzje dokumentujemy w ADR/PRD linkowanych z `PLAN.md`. Każdy kontekst biznesowy realizujemy jako mikroserwis (API, pricing, email itd.), komunikujący się przez jasno zdefiniowane kontrakty.

## Uzasadnienie
- Mniej kodu = krótsze wdrożenie nowych agentów.  
- Jawne kontrakty zmniejszają ryzyko błędów przy pracy równoległej.
- Dokumentowanie w ADR ogranicza utratę kontekstu wraz ze zmianami zespołu.

## Konsekwencje
+ Repozytorium można zrozumieć w kilka minut.  
+ Łatwiejsze refaktoryzacje i review przez agentów.  
– Więcej dyscypliny przy dodawaniu nowych modułów (pilnowanie limitów).
