# Power consumption calculator

## Architecture des données

Événements (timestamp, type, valeur):
    - POWER :
        - Intel RAPL (PKG, PP0, PP1, DRAM)
        - AMD RAPL
        - NVSMI
        - RAM (estimation)
        - Disques, périphériques (estimation)
        - Autres
        - TOTAL
    - UTILISATION :
        - CPU (procfs)
        - GPU (NVSMI)
        - RAM (procfs)

Configuration