
#include <stdbool.h>
#include <stddef.h>
#include <stdio.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

#include <errno.h>
#include <sys/mman.h>
#include <sys/types.h>
#include <sys/stat.h>
#include <fcntl.h>
#include <libelf.h>
#include <gelf.h>

struct str_section {
    Elf64_Half section_idx;
    
};

struct str_section str_tables[32];

void
print_symbols(Elf_Scn *scn, GElf_Shdr *shdr)
{
    int i;
    Elf_Data *edata = NULL;
    GElf_Sym sym;

    edata = elf_getdata(scn, edata);
    if (edata == NULL) {
        fprintf(stderr, "hrm edata?\n");
        return;
    }

    for (i = 0; i < shdr->sh_size / shdr->sh_entsize; i++) {
        if (gelf_getsym(edata, i, &sym) == NULL) {
            fprintf(stderr, "hrm gelf_getsym?\n");
            continue; 
        }

        if (ELF64_ST_TYPE(sym.st_info) == STT_FUNC) {
            printf(" sym val 0x%llx size %llu type %u"
                   "  section idx %u name %llu\n",
                   (unsigned long long)sym.st_value,
                   (unsigned long long)sym.st_size,
                   ELF64_ST_TYPE(sym.st_info),
                   sym.st_shndx,
                   (unsigned long long)sym.st_name);
        }
    }
}

static void
get_str_tables(fd)
{
    Elf *legolas;
    Elf_Scn *scn;
    int i;

    legolas = elf_begin(fd, ELF_C_READ, NULL);
    if (legolas == NULL) {
        fprintf(stderr, "damned elf\n");
        goto done;
    }

    scn = elf_nextscn(legolas, NULL);
    for (i = 0; scn != NULL; i++, scn = elf_nextscn(legolas, scn)) {
        GElf_Shdr shdr;

        if (gelf_getshdr(scn, &shdr) == NULL) {
            fprintf(stderr, "hrm gelf_getshdr?\n");            
            continue;
        }

#if 0
        if ((shdr.sh_type == SHT_SYMTAB) ||
            (shdr.sh_type == SHT_DYNSYM)) {
            printf("idx %u type 0x%x link %llu\n", i, shdr.sh_type,
                   (unsigned long long)shdr.sh_link);
            print_symbols(scn, &shdr);
        }
#endif
        if (shdr.sh_type == SHT_STRTAB) {
        }
    }
    elf_end(legolas);
 done: ;
}


int
main(int argc, char **argv)
{
    int fd = -1;
    uint8_t *ptr;
    struct stat s;

    fd = open(argv[1], O_RDONLY);
    if (fd < 0) {
        fprintf(stderr, "open %s %s\n", argv[1], strerror(errno));
        goto done;
    }

    if (fstat(fd, &s) != 0) {
        perror("fstat");
        goto done;
    }

    ptr = mmap(0, s.st_size, PROT_READ, 0, fd, 0);
    if (ptr == NULL) {
        fprintf(stderr, "mmap failure %s\n", strerror(errno));
        goto done;
    }

    /**
     * wowww elf_version does some initialization and without it
     * elf_begin always returns NULL!
     */
    if (elf_version(EV_CURRENT) == EV_NONE) {
        fprintf(stderr, "elf version %u\n", elf_version(EV_CURRENT));
        goto done;
    }

    get_str_tables(fd);

 done:
    if (fd >= 0) {
        close(fd);
    }
    return 0;
}
