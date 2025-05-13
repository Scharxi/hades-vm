# Hades-VM Opcode-Dokumentation

Diese Dokumentation beschreibt alle verfügbaren Opcodes der Hades-VM. Die VM verwendet einen Stack-basierten Ansatz mit einer 32-Bit-Befehlsarchitektur.

## Befehlsformat

Jeder Befehl ist 32 Bit (4 Bytes) lang und hat das folgende Format:
- Byte 0-2: Operanden (24 Bit)
- Byte 3: Opcode (8 Bit)

Bei Befehlen ohne Operanden werden die ersten 24 Bit ignoriert.

## Verfügbare Opcodes

| Hex  | Dezimal | Name          | Operanden | Beschreibung                                           |
|------|---------|---------------|-----------|--------------------------------------------------------|
| 0x01 | 1       | Add           | 0         | Addiert die obersten zwei Werte vom Stack              |
| 0x02 | 2       | Store         | 1         | Legt den angegebenen Wert auf den Stack               |
| 0x03 | 3       | Sub           | 0         | Subtrahiert den obersten Wert vom zweiten Wert        |
| 0x04 | 4       | LoadConstant  | 1         | Lädt eine Konstante auf den Stack                      |
| 0x05 | 5       | Multiply      | 0         | Multipliziert die obersten zwei Werte                  |
| 0x06 | 6       | Print         | 0         | Gibt den obersten Wert des Stacks aus                  |
| 0x07 | 7       | LoadMemory    | 1         | Lädt einen Wert aus dem Speicher auf den Stack         |
| 0x08 | 8       | StoreMemory   | 1         | Speichert den obersten Wert an der angegebenen Adresse |
| 0x09 | 9       | JumpIfZero    | 1         | Springt zur angegebenen Adresse, wenn oberster Wert 0  |
| 0x0A | 10      | Divide        | 0         | Dividiert den zweiten Wert durch den obersten Wert     |
| 0x0B | 11      | LoadFromRegion| 2         | Lädt einen Wert aus einer bestimmten Speicherregion    |
| 0x0C | 12      | StoreToRegion | 2         | Speichert in einer bestimmten Speicherregion           |
| 0x0D | 13      | Call          | 2         | Ruft eine Funktion auf                                |
| 0x0E | 14      | Return        | 0         | Kehrt von einer Funktion zurück                       |
| 0x0F | 15      | LoadLocal     | 1         | Lädt eine lokale Variable auf den Stack               |
| 0x10 | 16      | StoreLocal    | 1         | Speichert in eine lokale Variable                     |

## Detaillierte Beschreibung der Opcodes

### Add (0x01)
**Operanden:** Keine  
**Stack vorher:** `[..., wert1, wert2]`  
**Stack nachher:** `[..., ergebnis]`  

Nimmt die obersten zwei Werte vom Stack, addiert sie und legt das Ergebnis zurück auf den Stack. Unterstützt Integer- und Float-Werte. Beide Werte müssen vom gleichen Typ sein.

### Store (0x02)
**Operanden:** 1 (Wert)  
**Stack vorher:** `[...]`  
**Stack nachher:** `[..., wert]`  

Legt den im Operanden angegebenen Wert als Integer auf den Stack.

### Sub (0x03)
**Operanden:** Keine  
**Stack vorher:** `[..., wert1, wert2]`  
**Stack nachher:** `[..., ergebnis]`  

Subtrahiert den obersten Wert vom zweiten Wert auf dem Stack (wert1 - wert2) und legt das Ergebnis zurück auf den Stack. Unterstützt Integer- und Float-Werte.

### LoadConstant (0x04)
**Operanden:** 1 (Konstante)  
**Stack vorher:** `[...]`  
**Stack nachher:** `[..., konstante]`  

Lädt die im Operanden angegebene Konstante als Integer auf den Stack.

### Multiply (0x05)
**Operanden:** Keine  
**Stack vorher:** `[..., wert1, wert2]`  
**Stack nachher:** `[..., ergebnis]`  

Multipliziert die obersten zwei Werte auf dem Stack und legt das Ergebnis zurück auf den Stack. Unterstützt Integer- und Float-Werte.

### Print (0x06)
**Operanden:** Keine  
**Stack vorher:** `[..., wert]`  
**Stack nachher:** `[..., wert]` (unverändert)  

Gibt den obersten Wert des Stacks aus, ohne ihn zu entfernen. Die Ausgabe enthält auch Typ-Informationen (Integer, Float, Boolean, Reference).

### LoadMemory (0x07)
**Operanden:** 1 (Adresse)  
**Stack vorher:** `[...]`  
**Stack nachher:** `[..., wert]`  

Lädt einen Wert aus dem Speicher an der angegebenen Adresse und legt ihn auf den Stack.

### StoreMemory (0x08)
**Operanden:** 1 (Adresse)  
**Stack vorher:** `[..., wert]`  
**Stack nachher:** `[...]`  

Speichert den obersten Wert des Stacks an der angegebenen Adresse im Speicher und entfernt ihn vom Stack. Nur Integer-Werte können gespeichert werden.

### JumpIfZero (0x09)
**Operanden:** 1 (Sprungadresse)  
**Stack vorher:** `[..., wert]`  
**Stack nachher:** `[...]`  

Prüft, ob der oberste Wert des Stacks Null ist, und springt zur angegebenen Adresse, wenn dies der Fall ist. Der Wert wird vom Stack entfernt. Die Sprungadresse muss ein Vielfaches von 4 sein (Befehlsausrichtung).

### Divide (0x0A)
**Operanden:** Keine  
**Stack vorher:** `[..., wert1, wert2]`  
**Stack nachher:** `[..., ergebnis]`  

Dividiert den zweiten Wert durch den obersten Wert auf dem Stack (wert1 / wert2) und legt das Ergebnis zurück auf den Stack. Unterstützt Integer- und Float-Werte. Löst eine Panic bei Division durch Null aus.

### LoadFromRegion (0x0B)
**Operanden:** 2 (Region-ID, Offset)  
**Stack vorher:** `[...]`  
**Stack nachher:** `[..., wert]`  

Lädt einen Wert aus einer bestimmten Speicherregion mit dem angegebenen Offset und legt ihn auf den Stack. Die Region-IDs sind:
- 0: Code
- 1: Data
- 2: Stack
- 3: Heap
- 4: Constants
- 5: IO

### StoreToRegion (0x0C)
**Operanden:** 2 (Region-ID, Offset)  
**Stack vorher:** `[..., wert]`  
**Stack nachher:** `[...]`  

Speichert den obersten Wert des Stacks in einer bestimmten Speicherregion mit dem angegebenen Offset. Nur Integer-Werte können gespeichert werden. Die Region-IDs entsprechen denen von LoadFromRegion.

### Call (0x0D)
**Operanden:** 2 (Adresse, Anzahl lokaler Variablen)  
**Stack vorher:** `[..., param1, param2, ...]`  
**Stack nachher:** Stack-Frame mit lokalen Variablen  

Ruft eine Funktion an der angegebenen Adresse auf. Die Parameter müssen bereits auf dem Stack liegen. Der zweite Operand gibt an, wie viele lokale Variablen die Funktion hat (inklusive Parameter). Ein neuer Stack-Frame wird erstellt.

### Return (0x0E)
**Operanden:** Keine  
**Stack vorher:** `[..., rückgabewert]` (optional)  
**Stack nachher:** `[..., rückgabewert]` (wenn vorhanden)  

Kehrt von einer Funktion zurück zum Aufrufer. Wenn ein Wert auf dem Stack liegt, wird dieser als Rückgabewert verwendet und bleibt nach dem Entfernen des Stack-Frames erhalten.

### LoadLocal (0x0F)
**Operanden:** 1 (Index)  
**Stack vorher:** `[...]`  
**Stack nachher:** `[..., lokalerWert]`  

Lädt eine lokale Variable mit dem angegebenen Index aus dem aktuellen Stack-Frame auf den Stack.

### StoreLocal (0x10)
**Operanden:** 1 (Index)  
**Stack vorher:** `[..., wert]`  
**Stack nachher:** `[...]`  

Speichert den obersten Wert des Stacks in eine lokale Variable mit dem angegebenen Index im aktuellen Stack-Frame und entfernt ihn vom Stack.

## Speicherregionen

Die Hades-VM verwendet ein segmentiertes Speichermodell mit unterschiedlichen Regionen:

| Region-ID | Name      | Beschreibung                              |
|-----------|-----------|-------------------------------------------|
| 0         | Code      | Programm-Code und Instruktionen           |
| 1         | Data      | Statische/globale Daten                   |
| 2         | Stack     | Call-Stack und Ausführungsstack           |
| 3         | Heap      | Dynamisch allozierter Speicher            |
| 4         | Constants | Konstante Werte (schreibgeschützt)        |
| 5         | IO        | Memory-mapped I/O für Gerätezugriff       |

Jede Region hat spezifische Zugriffsrechte (Lesen, Schreiben, Ausführen), die vom VM-Layout bestimmt werden.

## Beispiele

### Beispiel 1: Addition zweier Zahlen
```
0x00 0x00 0x05 0x04  // LoadConstant 5
0x00 0x00 0x07 0x04  // LoadConstant 7
0x00 0x00 0x00 0x01  // Add
0x00 0x00 0x00 0x06  // Print
```

Dieses Programm lädt die Zahlen 5 und 7 auf den Stack, addiert sie und gibt das Ergebnis (12) aus.

### Beispiel 2: Bedingte Verzweigung
```
0x00 0x00 0x01 0x04  // LoadConstant 1
0x00 0x00 0x00 0x04  // LoadConstant 0
0x00 0x00 0x0C 0x09  // JumpIfZero 12 (springt zum Print-Befehl, wenn 0)
0x00 0x00 0x64 0x04  // LoadConstant 100 (wird übersprungen bei 0)
0x00 0x00 0x00 0x06  // Print
```

Dieses Programm lädt 1 und dann 0 auf den Stack, prüft ob der oberste Wert (0) Null ist und springt zur Adresse 12, wo der Wert 100 ausgegeben wird.

### Beispiel 3: Funktionsaufruf
```
// Hauptprogramm
0x00 0x00 0x05 0x04  // LoadConstant 5
0x00 0x00 0x0C 0x0D  // Call 12, 1 Parameter
0x00 0x00 0x00 0x06  // Print Ergebnis

// Funktion bei Adresse 12: Verdoppelt den Eingabewert
0x00 0x00 0x00 0x0F  // LoadLocal 0
0x00 0x00 0x00 0x0F  // LoadLocal 0
0x00 0x00 0x00 0x01  // Add
0x00 0x00 0x00 0x0E  // Return
```

Dieses Programm lädt die Zahl 5 auf den Stack, ruft die Funktion bei Adresse 12 auf, welche den Wert verdoppelt, und gibt dann das Ergebnis (10) aus. 