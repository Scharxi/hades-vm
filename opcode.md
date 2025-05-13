1. Arithmetische und logische Operationen
| Opcode | Name | Operanden | Beschreibung |
|--------|------|-----------|--------------|
| 0x11 | Modulo | 0 | Berechnet Modulo: stack[n-2] % stack[n-1] |
| 0x12 | Power | 0 | Potenzierung: stack[n-2] ^ stack[n-1] |
| 0x13 | BitAnd | 0 | Bitweise AND-Operation |
| 0x14 | BitOr | 0 | Bitweise OR-Operation |
| 0x15 | BitXor | 0 | Bitweise XOR-Operation |
| 0x16 | BitNot | 0 | Bitweise NOT-Operation (1er-Komplement) |
| 0x17 | ShiftLeft | 0 | Bitweise Linksverschiebung |
| 0x18 | ShiftRight | 0 | Bitweise Rechtsverschiebung |
2. Stack-Manipulationen
| Opcode | Name | Operanden | Beschreibung |
|--------|------|-----------|--------------|
| 0x19 | Dup | 0 | Dupliziert den obersten Stack-Wert |
| 0x1A | Swap | 0 | Vertauscht die obersten zwei Stack-Werte |
| 0x1B | Rotate | 1 | Rotiert die obersten n Elemente |
| 0x1C | Drop | 0 | Verwirft den obersten Stack-Wert |
| 0x1D | Over | 0 | Kopiert den zweiten Wert auf den Stack-Top |
| 0x1E | PickN | 1 | Kopiert den n-ten Wert auf den Stack-Top |
3. Vergleichsoperationen
| Opcode | Name | Operanden | Beschreibung |
|--------|------|-----------|--------------|
| 0x20 | Equal | 0 | Vergleicht zwei Werte auf Gleichheit |
| 0x21 | NotEqual | 0 | Vergleicht zwei Werte auf Ungleichheit |
| 0x22 | LessThan | 0 | Vergleicht ob stack[n-2] < stack[n-1] |
| 0x23 | GreaterThan | 0 | Vergleicht ob stack[n-2] > stack[n-1] |
| 0x24 | LessOrEqual | 0 | Vergleicht ob stack[n-2] <= stack[n-1] |
| 0x25 | GreaterOrEqual | 0 | Vergleicht ob stack[n-2] >= stack[n-1] |
4. Kontrollfluss-Erweiterungen
| Opcode | Name | Operanden | Beschreibung |
|--------|------|-----------|--------------|
| 0x30 | Jump | 1 | Unbedingter Sprung zur angegebenen Adresse |
| 0x31 | JumpIfNotZero | 1 | Springt, wenn oberster Wert nicht null ist |
| 0x32 | JumpIfNegative | 1 | Springt, wenn oberster Wert negativ ist |
| 0x33 | JumpIfPositive | 1 | Springt, wenn oberster Wert positiv ist |
| 0x34 | CallIndirect | 1 | Ruft Funktion über Adresse auf dem Stack auf |
| 0x35 | Switch | 1+ | Fallbasierte Verzweigung (operand = Anzahl Fälle) |
5. Typ-Konvertierungen
| Opcode | Name | Operanden | Beschreibung |
|--------|------|-----------|--------------|
| 0x40 | IntToFloat | 0 | Konvertiert Integer zu Float |
| 0x41 | FloatToInt | 0 | Konvertiert Float zu Integer (abrunden) |
| 0x42 | FloatToIntRound | 0 | Konvertiert Float zu Integer (runden) |
| 0x43 | FloatToIntCeil | 0 | Konvertiert Float zu Integer (aufrunden) |
| 0x44 | BoolToInt | 0 | Konvertiert Boolean zu Integer (0/1) |
6. Heap-Management
| Opcode | Name | Operanden | Beschreibung |
|--------|------|-----------|--------------|
| 0x50 | Allocate | 0 | Alloziert n Wörter auf dem Heap, liefert Referenz |
| 0x51 | Free | 0 | Gibt Heap-Speicher frei |
| 0x52 | LoadRef | 0 | Lädt einen Wert über eine Referenz |
| 0x53 | StoreRef | 0 | Speichert einen Wert über eine Referenz |
| 0x54 | ArrayLoad | 0 | Lädt aus Array: array_ref[index] |
| 0x55 | ArrayStore | 0 | Speichert in Array: array_ref[index] = value |
7. Floating-Point-Operationen
| Opcode | Name | Operanden | Beschreibung |
|--------|------|-----------|--------------|
| 0x60 | Sqrt | 0 | Berechnet die Quadratwurzel |
| 0x61 | Sin | 0 | Berechnet den Sinus |
| 0x62 | Cos | 0 | Berechnet den Cosinus |
| 0x63 | Tan | 0 | Berechnet den Tangens |
| 0x64 | Exp | 0 | Berechnet e^x |
| 0x65 | Log | 0 | Berechnet den natürlichen Logarithmus |
| 0x66 | Pow | 0 | Potenzierung für Float-Werte |
8. String-Operationen
| Opcode | Name | Operanden | Beschreibung |
|--------|------|-----------|--------------|
| 0x70 | LoadString | 1 | Lädt einen String aus der Konstanten-Region |
| 0x71 | PrintString | 0 | Gibt einen String aus |
| 0x72 | StringConcat | 0 | Konkateniert zwei Strings |
| 0x73 | StringLength | 0 | Berechnet die Länge eines Strings |
| 0x74 | StringCompare | 0 | Vergleicht zwei Strings |
9. I/O-Operationen
| Opcode | Name | Operanden | Beschreibung |
|--------|------|-----------|--------------|
| 0x80 | ReadInteger | 0 | Liest einen Integer von der Standardeingabe |
| 0x81 | ReadFloat | 0 | Liest einen Float von der Standardeingabe |
| 0x82 | ReadString | 0 | Liest einen String von der Standardeingabe |
| 0x83 | PrintChar | 0 | Gibt ein Zeichen aus (ASCII/Unicode) |
| 0x84 | FileOpen | 0 | Öffnet eine Datei, liefert Handle |
| 0x85 | FileClose | 0 | Schließt eine Datei |
| 0x86 | FileRead | 0 | Liest aus einer Datei |
| 0x87 | FileWrite | 0 | Schreibt in eine Datei |
10. Debugging-Unterstützung
| Opcode | Name | Operanden | Beschreibung |
|--------|------|-----------|--------------|
| 0x90 | Breakpoint | 0 | Setzt einen Breakpoint für Debugger |
| 0x91 | StackTrace | 0 | Gibt einen Stack-Trace aus |
| 0x92 | PrintMemoryDump | 1 | Gibt einen Speicherauszug aus |
| 0x93 | Assert | 0 | Prüft Bedingung und bricht bei Fehler ab |
| 0x94 | PrintStackState | 0 | Gibt aktuellen Stack-Zustand aus |
11. Parallele Verarbeitung
| Opcode | Name | Operanden | Beschreibung |
|--------|------|-----------|--------------|
| 0xA0 | SpawnThread | 1 | Startet einen neuen Thread mit Funktion |
| 0xA1 | JoinThread | 0 | Wartet auf Thread-Beendigung |
| 0xA2 | MutexLock | 0 | Sperrt einen Mutex |
| 0xA3 | MutexUnlock | 0 | Entsperrt einen Mutex |
| 0xA4 | AtomicLoad | 0 | Atomares Laden eines Werts |
| 0xA5 | AtomicStore | 0 | Atomares Speichern eines Werts |
12. Erweitertes Speichermodell
| Opcode | Name | Operanden | Beschreibung |
|--------|------|-----------|--------------|
| 0xB0 | DefineStruct | 1+ | Definiert einen Struct-Typ (operand = Anzahl Felder) |
| 0xB1 | LoadField | 1 | Lädt ein Feld aus einem Struct |
| 0xB2 | StoreField | 1 | Speichert ein Feld in einem Struct |
| 0xB3 | CreateArray | 1 | Erstellt ein Array mit bestimmter Größe |
| 0xB4 | GetArrayLen | 0 | Ermittelt die Länge eines Arrays |
13. Module und Bibliotheken
| Opcode | Name | Operanden | Beschreibung |
|--------|------|-----------|--------------|
| 0xC0 | ImportModule | 1 | Importiert ein Modul |
| 0xC1 | ExportSymbol | 1 | Exportiert ein Symbol |
| 0xC2 | LinkSymbol | 1 | Verknüpft ein Symbol zur Laufzeit |
| 0xC3 | FindSymbol | 1 | Sucht ein Symbol in geladenen Modulen |
14. Ausnahmebehandlung
| Opcode | Name | Operanden | Beschreibung |
|--------|------|-----------|--------------|
| 0xD0 | TryBegin | 1 | Startet einen Try-Block (operand = catch-Adresse) |
| 0xD1 | TryEnd | 0 | Beendet einen Try-Block |
| 0xD2 | Throw | 0 | Wirft eine Ausnahme |
| 0xD3 | Rethrow | 0 | Wirft eine gefangene Ausnahme erneut |