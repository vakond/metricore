# metricore

### Simple high-load HTTP service to gather metrics

This **toy project** is a simple HTTP server to gather "events" (small packets of information). It explores some Rust idioms and libraries. The project was created with idea of simplicity in mind.

*    Build:

         cargo build --release

*    Usage:

         metricore

*    Shutdown:

         Ctrl+C or killall metricore

*    Simple tests (requires httpie utility):

         http PUT localhost:8080 event=A
         http POST localhost:8080 event=B

*    Smoke test:

         smoke_test_linux.sh

*    Stress test:

         stress_test_linux.sh
