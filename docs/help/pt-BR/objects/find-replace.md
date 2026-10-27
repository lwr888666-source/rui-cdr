# Localizar e substituir

O painel [[docker.find_replace]] (<kbd>Ctrl+F</kbd>) localiza texto no desenho,
e o substitui, ou localiza objetos pelo nome. Abra-o com
[[menu.edit > menu.edit.find_and_replace]].

![O painel Localizar e substituir](../../img/objects-find-replace.png)

## Localizar e substituir texto {#text}

1. Escolha [[dialog.find_text]].
2. Digite o que procurar em [[dialog.find]], e o texto novo em
   [[dialog.replace_with]].
3. Marque [[dialog.match_case]] para distinguir maiúsculas de minúsculas, e
   [[dialog.whole_word]] para ignorar o texto quando ele é parte de uma palavra
   maior.
4. Clique em [[dialog.find_next]] para selecionar o próximo objeto de texto que
   o contém, ou em [[dialog.find_all]] para selecionar todos.
5. Clique em [[dialog.replace]] para substituir no texto selecionado, ou em
   [[dialog.replace_all]] para substituir em todo lugar.

O painel diz quantos foram encontrados.

## Localizar objetos {#objects}

Escolha [[dialog.find_objects]], digite parte de um nome, ou um tipo de objeto
como "Rectangle", e clique em [[dialog.find_next]] ou [[dialog.find_all]]. A
busca cobre a página atual. Para dar nome a objetos, veja
[Renomear objetos e camadas](objects-and-layers.md#rename).
