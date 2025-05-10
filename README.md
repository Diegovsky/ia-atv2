# Como rodar
Para que não seja necessário que você tenha instalado um ambiente de desenvolvimento rust, incluí binários pré compilados para linux-x86_64 na pasta `bin/`.
Dito isso, para saber como rodar o programa, basta chamar sua função `help`:
```
tsp_sa --help
```

Obtive resultados bons com as flags `-t 80 -a 0.99 -i 10`
# Como compilar (opcional)
Você vai precisar dos programas `rustc` e `cargo`. Normalmente podem ser instalados nas distribuições linux pelo pacote `rust`.

Basta executar `cargo build`, e após isso, os programas estarão localizados em `target/debug/`.
