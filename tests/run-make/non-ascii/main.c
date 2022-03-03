#include "../check.h"
#include <stdio.h>
#include <string.h>

int main(int argc, char *argv[]) {
  // 4-byte sequences
  check_file_line("/mnt/emoji-🐼-🐶-🐱.txt", "Cute animals 🐭\n");
  // 3-byte sequences
  check_file_line("/mnt/こんにちは.txt", "日本語（にほんご、にっぽんご）は、主に日本国内や日本人同士の間で使われている言語である。\n");
  // 2-byte sequences
  check_file_line("/mnt/Привет.txt", "Ру́сский язы́к один из восточнославянских языков, национальный язык русского народа.\n");
  return 0;
}
