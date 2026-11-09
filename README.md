# AlfoPlus Configurator

Windows aplikace, která pomáhá připojit počítač k anténě AlfoPlus v záchranném režimu.

## Funkce

- Zobrazí síťové adaptéry, jejich stav linku a aktuální IPv4 adresy.
- Na vybraném adaptéru přidá dočasnou adresu `172.20.253.14/16`; ostatní IPv4 adresy, DHCP, DNS, bránu, routy a IPv6 ponechá beze změny.
- Průběžně zobrazuje stav adaptéru a čeká na tři po sobě jdoucí úspěšná ověření: ping antény `172.20.253.13` a TCP spojení na port 80.
- Výchozí prohlížeč otevře až po ověření dostupnosti webového rozhraní.
- Při dokončení odstraní pouze adresu přidanou aplikací. Adresu, která na adaptéru byla už před spuštěním, ponechá.

## Použití

1. Připojte Ethernet port počítače do LAN/DATA portu PoE injektoru a anténu do PoE portu.
2. Spusťte aplikaci jako správce a vyberte správný fyzický síťový adaptér.
3. Spusťte připojení. Po potvrzení dočasné adresy zapněte napájení PoE injektoru.
4. Po ověření pingů a portu 80 aplikace otevře webové rozhraní antény.
5. Po dokončení konfigurace zvolte dokončení průvodce, aby se odstranila dočasná adresa vytvořená aplikací.

## Požadavky a omezení

- Windows x64 a oprávnění správce pro změnu síťového adaptéru.
- Průvodce používá adresy `172.20.253.14/16` pro počítač a `172.20.253.13` pro anténu.
- Konfigurace antény se provádí ručně v jejím webovém rozhraní. Aplikace neprovádí přihlášení, změny přes Telnet/SSH ani automatické úpravy IP antény, VLAN nebo rádiových parametrů.
- Prohlížeč otevře HTTP adresu `http://172.20.253.13` po úspěšném ping testu a dostupném TCP portu 80.

## Stažení

Release `v0.1.0` obsahuje samostatný Windows x64 EXE a přenosný ZIP s aplikací, README a licencemi. Aplikaci není potřeba instalovat.

## Build ze zdrojových kódů

Nainstalujte stable Rust a Windows MSVC target, poté spusťte:

```powershell
rustup target add x86_64-pc-windows-msvc
cargo build --release --target x86_64-pc-windows-msvc
```

Výsledný soubor je `target/x86_64-pc-windows-msvc/release/alfoplus-configurator.exe`.

## Licence

Licencováno pod MIT nebo Apache-2.0, dle vlastního výběru.
