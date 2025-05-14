# Speicherverwaltung in der Hades-VM

## Überblick

Die Hades-VM implementiert ein System zur dynamischen Speicherverwaltung, das es Programmen ermöglicht, während der Laufzeit Speicher anzufordern und freizugeben. Dieses Dokument beschreibt die Implementierung der Speicherverwaltung, die verfügbaren VM-Befehle für Speicheroperationen und gibt Beispiele für deren Verwendung.

## Speicherarchitektur

Die Hades-VM verwendet eine segmentierte Speicherarchitektur mit verschiedenen Regionen:

- **Code-Region**: Enthält die Programminstruktionen
- **Data-Region**: Enthält statische Daten
- **Stack-Region**: Wird für den Call-Stack und lokale Variablen verwendet
- **Heap-Region**: Wird für dynamisch allozierte Daten verwendet
- **Constants-Region**: Enthält Konstanten
- **IO-Region**: Dient für memory-mapped I/O

Die dynamische Speicherverwaltung betrifft hauptsächlich die Heap-Region, die für die Laufzeitallokation von Daten reserviert ist.

## Implementierung der Speicherverwaltung

### Allocation Bitmap

Die VM verwendet eine Bitmap-basierte Speicherverwaltung, die direkt im Heap-Speicher selbst gespeichert wird. Die Bitmap enthält:

1. Einen Marker `0xA110C` am Anfang des Heaps
2. Eine Zählung der Anzahl der Blöcke im Heap
3. Einträge für jeden Block mit folgender Struktur:
   - Blockadresse (Wort 1)
   - Blockgröße (Wort 2)
   - Allokationsflag (0 = frei, 1 = alloziert) (Wort 3)

### Allokationsalgorithmus

Der Allokationsalgorithmus verwendet eine First-Fit-Strategie:

1. Durchsuchen aller vorhandenen freien Blöcke nach einem passenden Block
2. Wenn ein freier Block größer als der angeforderte ist, wird er aufgeteilt (Split)
3. Wenn kein freier Block gefunden wird, wird ein neuer Block am Ende der Bitmap erstellt

### Deallocations-Prozess

Der Deallocations-Prozess:

1. Findet den Block mit der angegebenen Adresse
2. Markiert den Block als frei, indem das Allokationsflag auf 0 gesetzt wird

## VM-Befehle für die Speicherverwaltung

### Alloc (0x50)

Fordert einen Speicherblock mit der angegebenen Größe an.

- **Operanden**: 1 (Größe des zu allozierenden Speicherblocks in Wörtern)
- **Stack-Ein**: Keine
- **Stack-Aus**: Referenz zur allozierten Adresse (oder 0 bei Fehlschlag)

### Free (0x51)

Gibt einen zuvor allozierten Speicherblock frei.

- **Operanden**: Keine
- **Stack-Ein**: Referenz zur Adresse des freizugebenden Blocks
- **Stack-Aus**: Boolean-Wert (true = erfolgreich, false = fehlgeschlagen)

### LoadHeap (0x52)

Liest einen Wert aus dem Heap-Speicher.

- **Operanden**: Keine
- **Stack-Ein**: Referenz zur Basisadresse, Offset
- **Stack-Aus**: Gelesener Integer-Wert

### StoreHeap (0x53)

Schreibt einen Wert in den Heap-Speicher.

- **Operanden**: Keine
- **Stack-Ein**: Referenz zur Basisadresse, Offset, zu schreibender Wert
- **Stack-Aus**: Keine

### MemSet (0x54)

Initialisiert einen Speicherbereich mit einem bestimmten Wert.

- **Operanden**: Keine
- **Stack-Ein**: Referenz zur Basisadresse, Anzahl der zu initialisierenden Wörter, Initialwert
- **Stack-Aus**: Keine

## Fehlerbehandlung

Die Speicherverwaltung implementiert verschiedene Sicherheitsmaßnahmen:

- Null-Pointer-Checks für alle Speicherzugriffe
- Typ-Checks für alle Stack-Werte
- Stack-Underflow-Prüfungen
- Überprüfung der Schreib-/Leseberechtigungen für Speicherzugriffe
- Fehlerberichterstattung durch Exceptions

## Codebeispiele

### Beispiel 1: Speicher allozieren und initialisieren

```
// Alloziere 10 Wörter Speicher
LoadConstant 10
Alloc

// Initialisiere den Speicher mit dem Wert 42
// Stack: [reference]
Dup             // Dupliziere die Referenz
LoadConstant 10 // Anzahl der Wörter
LoadConstant 42 // Initialwert
MemSet          // Initialisiere den Speicher
```

### Beispiel 2: Werte im Heap lesen und schreiben

```
// Alloziere Speicher und behalte die Referenz
LoadConstant 5
Alloc
// Stack: [reference]

// Schreibe den Wert 123 an Offset 2
Dup             // Dupliziere die Referenz
LoadConstant 2  // Offset
LoadConstant 123 // Wert
StoreHeap

// Lese den Wert an Offset 2 zurück
Dup             // Dupliziere die Referenz
LoadConstant 2  // Offset
LoadHeap
// Stack: [reference, 123]

// Gib den Wert aus
Print           // Gibt "Integer: 123" aus

// Gib den Speicher frei
Free
// Stack: [boolean] - Erfolg der Freigabe
```

### Beispiel 3: Dynamisches Array

```
// Alloziere Speicher für ein Array mit 5 Elementen
LoadConstant 5
Alloc
// Stack: [reference]

// Schreibe die Länge des Arrays an die erste Position
Dup             // Dupliziere die Referenz
LoadConstant 0  // Offset 0
LoadConstant 5  // Länge 5
StoreHeap

// Schreibe Werte ins Array (Index 1 bis 5)
Dup
LoadConstant 1  // Index 1
LoadConstant 10 // Wert 10
StoreHeap

Dup
LoadConstant 2  // Index 2
LoadConstant 20 // Wert 20
StoreHeap

Dup
LoadConstant 3  // Index 3
LoadConstant 30 // Wert 30
StoreHeap

Dup
LoadConstant 4  // Index 4
LoadConstant 40 // Wert 40
StoreHeap

Dup
LoadConstant 5  // Index 5
LoadConstant 50 // Wert 50
StoreHeap

// Jetzt haben wir ein Array [5, 10, 20, 30, 40, 50] im Heap
// wobei das erste Element die Länge des Arrays ist
```

## Leistungsaspekte

### Fragmentierung

Die aktuelle Implementierung kann zur Fragmentierung des Heap-Speichers führen. Um dieses Problem zu mildern, könnten folgende Strategien implementiert werden:

- Coalescing benachbarter freier Blöcke bei Deallokation
- Verwendung einer Best-Fit- oder Worst-Fit-Strategie anstelle von First-Fit
- Implementierung eines Kompaktierungsalgorithmus

### Speichereffizienz

Die Bitmap-basierte Implementierung hat einen gewissen Overhead für die Verwaltungsdaten. Für jeden allozierten Block werden 3 Wörter für Metadaten verwendet.

## Zukünftige Erweiterungen

Mögliche Erweiterungen für die Speicherverwaltung der Hades-VM:

1. **Garbage Collection**: Automatische Speicherbereinigung für nicht mehr referenzierte Objekte
2. **Coalescing**: Automatisches Zusammenführen benachbarter freier Blöcke
3. **Alternative Allokationsstrategien**: Best-Fit oder Next-Fit-Algorithmen
4. **Speicherkompaktierung**: Umordnen von Speicherblöcken zur Reduzierung der Fragmentierung
5. **Regionbasierte Speicherverwaltung**: Allokation in Regionen mit automatischer Freigabe
6. **Debugging-Tools**: Werkzeuge zur Identifizierung von Speicherlecks oder Pufferüberläufen 