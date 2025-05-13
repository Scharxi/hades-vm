# Memory Management der Hades-VM

Diese Dokumentation beschreibt das Memory Management-System der Hades-VM, ein segmentiertes Speichermodell mit Zugriffskontrollen, das für Flexibilität und Sicherheit konzipiert wurde.

## Überblick

Die Hades-VM implementiert ein segmentiertes Speichermodell, bei dem der Speicher in verschiedene funktionale Regionen mit spezifischen Zugriffsrechten unterteilt ist. Dieses Design bietet mehrere Vorteile:

1. **Speicherschutz**: Verhindert unbeabsichtigten oder unbefugten Zugriff auf Speicherbereiche
2. **Isolation**: Trennt Code von Daten und Stack von Heap
3. **Flexibilität**: Unterstützt verschiedene Speicherlayouts für unterschiedliche Anwendungsfälle

Der Speicher der VM besteht aus 32-Bit-Wörtern (i32) und wird über ein einheitliches API für Lese- und Schreiboperationen angesprochen.

## Speicherregionen

Die VM unterstützt die folgenden vordefinierten Speicherregiontypen:

| Region     | Beschreibung                                      | Typische Zugriffsrechte       |
|------------|---------------------------------------------------|-------------------------------|
| Code       | Programmanweisungen und ausführbarer Code         | Lesen, Ausführen              |
| Data       | Globale und statische Variablen                   | Lesen, Schreiben              |
| Stack      | Call-Stack und lokale Variablen                   | Lesen, Schreiben              |
| Heap       | Dynamisch allozierter Speicher                    | Lesen, Schreiben              |
| Constants  | Konstante Werte und unveränderliche Daten         | Nur Lesen                     |
| IO         | Memory-mapped I/O für Geräteinteraktion           | Lesen, Schreiben              |

## Zugriffsrechte

Jede Speicherregion kann über eine Kombination der folgenden Zugriffsrechte verfügen:

- **Read**: Erlaubt das Lesen von Daten aus der Region
- **Write**: Erlaubt das Schreiben von Daten in die Region
- **Execute**: Erlaubt die Ausführung von Code aus der Region

Diese Rechte werden bei jedem Speicherzugriff überprüft, um unerlaubte Operationen zu verhindern und die Speicherintegrität zu gewährleisten.

## Speicherlayouts

Die VM bietet verschiedene vordefinierte Speicherlayouts für unterschiedliche Anwendungsfälle:

### Standard-Layout

Dieses Layout ist für allgemeine Anwendungen gedacht und teilt den Speicher wie folgt auf:

- **Code (20%)**: Programmanweisungen (Lesen, Ausführen)
- **Constants (10%)**: Konstante Werte (Nur Lesen)
- **Data (20%)**: Globale Variablen (Lesen, Schreiben)
- **Stack (25%)**: Call-Stack und lokale Variablen (Lesen, Schreiben)
- **Heap (25%)**: Dynamische Speicherallokation (Lesen, Schreiben)

Grafische Darstellung:
```
+---------------+
| Code (20%)    | 0
+---------------+
| Constants (10%)| 20%
+---------------+
| Data (20%)    | 30%
+---------------+
| Stack (25%)   | 50%
+---------------+
| Heap (25%)    | 75%
+---------------+ 100%
```

### Embedded-Layout

Dieses Layout simuliert die Speichereinschränkungen eingebetteter Systeme und ist wie folgt aufgeteilt:

- **Flash (50%)**: Unterteilt in Code und Constants
  - **Code (25%)**: Programmanweisungen (Lesen, Ausführen)
  - **Constants (25%)**: Konstante Werte (Nur Lesen)
- **RAM (40%)**: Unterteilt in Data und Stack
  - **Data (20%)**: Globale Variablen (Lesen, Schreiben)
  - **Stack (20%)**: Call-Stack und lokale Variablen (Lesen, Schreiben)
- **IO (10%)**: Memory-mapped I/O (Lesen, Schreiben)

Grafische Darstellung:
```
+---------------+
| Code (25%)    | 0
+---------------+
| Constants (25%)| 25%
+---------------+
| Data (20%)    | 50%
+---------------+
| Stack (20%)   | 70%
+---------------+
| IO (10%)      | 90%
+---------------+ 100%
```

### Test-Layout

Das Test-Layout folgt der Struktur des Standard-Layouts, gewährt jedoch allen Regionen vollständige Zugriffsrechte (Lesen, Schreiben, Ausführen), um Tests zu vereinfachen.

## Speicherzugriff

Die VM bietet verschiedene Methoden für den Speicherzugriff:

### Globaler Adressraum

- **read(address)**: Liest einen Wert von einer absoluten Adresse
- **write(address, value)**: Schreibt einen Wert an eine absolute Adresse
- **execute(address)**: Führt Code an einer absoluten Adresse aus (liest eine Instruktion)

Beispiel:
```rust
// Lesen von Adresse 100
memory.read(100);

// Schreiben des Werts 42 an Adresse 200
memory.write(200, 42);
```

### Regionbasierter Zugriff

- **read_from_region(region_type, offset)**: Liest einen Wert aus einer bestimmten Region an einem Offset
- **write_to_region(region_type, offset, value)**: Schreibt einen Wert in eine bestimmte Region an einem Offset

Beispiel:
```rust
// Lesen aus der Data-Region an Offset 50
memory.read_from_region(MemoryRegionType::Data, 50);

// Schreiben in die Stack-Region an Offset 10
memory.write_to_region(MemoryRegionType::Stack, 10, 42);
```

## Fehlerbehandlung

Das Memory Management-System implementiert umfassende Fehlerprüfungen, einschließlich:

1. **Adressvalidierung**: Überprüft, ob Adressen innerhalb der Speichergrenzen liegen
2. **Regionüberprüfung**: Stellt sicher, dass Adressen zu definierten Regionen gehören
3. **Zugriffsprüfung**: Verifiziert, dass die angeforderte Operation (Lesen, Schreiben, Ausführen) für die Region erlaubt ist

Bei Speicherzugriffsfehlern werden beschreibende Fehlermeldungen zurückgegeben, die bei der Diagnose von Programmproblemen helfen.

## Speichergröße und Erweiterung

Der Speicher wird mit einer anfänglichen Größe erstellt, kann aber bei Bedarf mit der `resize`-Methode erweitert werden. Die Verkleinerung des Speichers wird nicht unterstützt, um Datenverlust zu vermeiden.

```rust
// Speicher auf 10000 Wörter erweitern
memory.resize(10000);
```

## Memory Map

Für Debugging-Zwecke bietet die VM eine `memory_map`-Funktion, die eine Textdarstellung des Speicherlayouts liefert, einschließlich:

- Startadresse jeder Region
- Endadresse jeder Region
- Größe in Wörtern
- Regionstyp
- Zugriffsrechte (RWX-Format)
- Beschreibung

## Benutzerdefinierte Layouts

Neben den vordefinierten Layouts können benutzerdefinierte Speicherlayouts erstellt werden, indem Regionen manuell definiert werden:

```rust
// Speicher mit 1000 Wörtern erstellen
let mut memory = SegmentedMemory::new(1000);

// Benutzerdefinierte Code-Region definieren
memory.define_region(MemoryRegion::new(
    MemoryRegionType::Code,
    0,        // Startadresse
    400,      // Größe
    vec![AccessPermission::Read, AccessPermission::Execute],
    Some("Benutzerdefinierter Code-Bereich".to_string()),
));

// Weitere Regionen nach Bedarf definieren...
```

## Speicherabstraktion

Das Memory Management-System der Hades-VM abstrahiert von physischen Speicherdetails und bietet eine konsistente Schnittstelle für alle Speicheroperationen. Dies ermöglicht es, die VM auf verschiedenen Plattformen zu implementieren und verschiedene Speichertechnologien zu unterstützen, während die gleiche Programmierschnittstelle beibehalten wird.

## Technische Implementierung

Der segmentierte Speicher wird durch drei Hauptkomponenten implementiert:

1. **data**: Ein Vektor von i32-Werten, der den eigentlichen Speicherinhalt darstellt
2. **regions**: Eine Hashtabelle, die Regionstypen auf Regionsdefinitionen abbildet
3. **region_map**: Ein Vektor, der für jede Adresse den zugehörigen Regionstyp speichert (für schnelle Lookups)

Diese Struktur ermöglicht sowohl effiziente Speicherzugriffe als auch robuste Zugriffskontrolle. 