# ADR 004: Skalowanie i gotowość na Kubernetes

## Kontekst
Startujemy na pojedynczym VPS (~10 USD), ale chcemy zachować ścieżkę migracji „bez zaskoczeń” do Kubernetes, gdy wzrośnie ruch, zespół lub pojawi się potrzeba CI/CD na klastrze.

## Decyzja
Projektujemy wszystkie serwisy w duchu „Kubernetes-ready”: rootless kontenery, konfiguracja przez zmienne środowiskowe, brak lokalnego stanu w binarkach, jasny kontrakt na storage i kolejkę. Na MVP używamy docker-compose, lecz utrzymujemy katalog `infra/k8s/` z manifestami (Deployment/StatefulSet/Service/Ingress) oraz opisem wymagań. Migracja nastąpi dopiero gdy to biznesowo uzasadnione.

## Uzasadnienie
- 12-factor + kontenery pozwalają wrzucić serwis na K8s praktycznie natychmiast (Deployment + Service).  
- Trzymanie Postgresa/Redis/S3 jako osobnych usług (lub managed) zmniejsza złożoność migracji – w klastrze zostaną tylko stateless workloady.  
- Przygotowanie manifestów wcześniej daje przewidywalność dla agentów i skraca czas późniejszego wdrożenia.  
- Rootless obrazy zwiększają bezpieczeństwo zarówno na VPS, jak i w klastrze.

## Plan „just in case”
1. **Kontenery**: każdy serwis ma Dockerfile budujący rootless image (np. `FROM gcr.io/distroless/static`). Artefakty publikujemy do prywatnego registry.  
2. **Konfiguracja**: wyłącznie zmienne środowiskowe + pliki `.env`. Brak plików konfiguracyjnych w repo poza template’ami.  
3. **Infra/k8s/**: przygotować szkic manifestów (Deployment + Service dla API, pricing, email; CronJob jeśli potrzeba). Dla Postgresa opisujemy opcje: managed DB lub StatefulSet z PVC.  
4. **Kolejka / messaging**: Redis/NATS jako osobny Helm chart; serwisy korzystają przez zmienne `QUEUE_URL`.  
5. **Secrets**: używać K8s Secrets / SOPS; już teraz trzymać klucze poza repo.  
6. **Observability**: dodać `/metrics` (Prometheus), `/healthz`, `/readyz`. W manifestach przewidzieć ServiceMonitor (opcjonalne).  
7. **HPA-ready**: serwisy eksportują CPU/memory metrics → w przyszłości włączamy Horizontal Pod Autoscaler.  
8. **CI/CD**: pipelines generują obrazy + `kubectl apply`/Helm (gdy pojawi się klaster).

## Konsekwencje
+ MVP pozostaje lekkie (docker-compose), ale migracja do K8s sprowadzi się do odpalenia gotowych manifestów.  
+ Każdy nowy serwis ma oczywiste miejsce w klastrze (Deployment + Service).  
– Koszt początkowy: przygotowanie manifestów i utrzymanie rootless obrazów.  
– Trzeba pilnować spójności konfiguracji (env vars) między docker-compose a manifestami.

## Status
Proposed
