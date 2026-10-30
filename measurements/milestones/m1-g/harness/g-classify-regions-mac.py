# M1 phase G: sorts g-regions-mac output (one macOS map entry per line) into kinds:
# the dyld shared region, malloc/libSystem, thread stacks, Rust alternate signal stacks,
# the executable image. Usage: python3 -I g-classify-regions-mac.py REGIONS.txt...
import sys,re,collections
# Mach VM user tags (mach/vm_statistics.h, VM_MEMORY_*): 1 malloc, 2 malloc_small, 7 malloc_tiny,
# 11 malloc_nano?, 22 ?, 30 stack, 31 shared pmap/guard, 32 dyld shared cache, 35 dyld shared
# region data, 73 ?, 78 ?
def cls(addr, kb, tag, share, prot, first_img):
    a=int(addr,16)
    if 0x180000000 <= a < 0xfc0000000: return 'dyld shared region (submaps and their private data)'
    if tag==31: return 'shared-region guard reservations'
    if tag==30: return 'thread stacks and guards'
    if tag in (1,2,7,11,22,73,78) or (tag==0 and share==3 and prot=='0/0'): return 'malloc/libsystem (zones, metadata, reservations)'
    if tag==0 and share==3 and kb in (128,16): return 'Rust alternate signal stacks and guards'
    if first_img <= a < first_img+0x200000: return 'the executable image'
    return 'other'
for f in sys.argv[1:]:
    rows=[l.split() for l in open(f)]
    first=int(rows[0][1],16)
    c=collections.Counter()
    for r in rows:
        tag=int(r[4].split('=')[1]) if r[4].startswith('tag=') and r[4]!='tag=' else int(r[5])
        share=int([x for x in r if x.startswith('share=')][0].split('=')[1]); prot=[x for x in r if x.startswith('prot=')][0].split('=')[1]
        c[cls(r[1],int(r[2]),tag,share,prot,first)]+=1
    shared=c['dyld shared region (submaps and their private data)']+c['shared-region guard reservations']
    print(f.split('/')[-1], sum(c.values()), 'own=%d' % (sum(c.values())-shared), dict(sorted(c.items(), key=lambda x:-x[1])))
