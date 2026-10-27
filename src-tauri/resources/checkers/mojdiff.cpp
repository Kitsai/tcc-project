#include "testlib.h"
#include <cctype>
#include <string>
#include <vector>
using namespace std;

// Aproxima o comparador padrão do MOJ (lang/compare.sh: cascata de `diff -i -b -B -w`).
// Aceita se as saídas coincidirem ignorando maiúsculas/minúsculas, qualquer espaço
// em branco e linhas em branco; a estrutura de linhas é preservada.
// Limitação: o MOJ distingue "Accepted" de "Accepted,PE" (só espaçamento difere);
// aqui ambos são "ok".
static vector<string> readNormalized(InStream &in) {
  vector<string> lines;
  while (!in.eof()) {
    string raw = in.readString();
    if (raw.empty() && in.eof())
      break;
    string norm;
    for (char c : raw)
      if (!isspace(static_cast<unsigned char>(c)))
        norm += static_cast<char>(tolower(static_cast<unsigned char>(c)));
    if (!norm.empty())
      lines.push_back(norm);
  }
  return lines;
}

int main(int argc, char *argv[]) {
  setName("compare like MOJ default diff (ignore case, whitespace, blank lines)");
  registerTestlibCmd(argc, argv);

  vector<string> expected = readNormalized(ans);
  vector<string> found = readNormalized(ouf);

  size_t n = min(expected.size(), found.size());
  for (size_t i = 0; i < n; i++)
    if (expected[i] != found[i])
      quitf(_wa, "line %d differs - expected: '%s', found: '%s'", (int)i + 1,
            expected[i].c_str(), found[i].c_str());
  if (expected.size() != found.size())
    quitf(_wa, "expected %d non-empty lines, found %d", (int)expected.size(),
          (int)found.size());

  quitf(_ok, "%d lines", (int)expected.size());
}
