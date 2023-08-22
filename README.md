# Rinha de Backend

## Introdução

Esta é a minha submissão à 1ª Rinha de Backend.
Se trata de uma competição organizada pela comunidade de desenvolvimento brasileira no Twitter (@rinhadebackend), liderada pelo @zanfranceschi.
A ideia é construir uma API simples, de acordo com uma especificação e provisionar a infra que suporte a maior carga possível.
Mas há restrições: deve haver 2 instâncias da aplicação, balanceadas pelo Nginx.
Tudo isso, mais o banco de dados, deve rodar em docker-compose, com apenas 1.5 CPU e 3 GB de memória para distribuir entre os serviços.
Para mais detalhes, olhe [aqui](https://github.com/zanfranceschi/rinha-de-backend-2023-q3)

## Backend

É uma aplicação backend básica, escrita em Rust.
O framework web é o [actix-web](https://actix.rs/).
Por questões de tempo, e por não ser relevante à competição em si, não inclui muitos detalhes que consideraria importantes numa aplicação profissional, como padronizar o formato das respostas de erro.
Também por ser uma aplicação muito pequena, ela basicamente não possui arquitetura definida.

O que fiz questão de incluir, porque não considero nenhuma aplicação backend pronta sem isso, e já tive traumas com a sua ausência, é observabilidade.
A aplicação está configurada para coletar traços e eventos no padrão Opentelemetry.
Incluindo a variável de ambiente correspondente, esses dados são enviados à Honeycomb.
A análise dos traços, aliás, ajudou a identificar onde estavam os gargalos durante a otimização da infra.
Mas, a aplicação precisar ser executada como parte da competição implica que é preciso também informar as variáveis de ambiente (que estão no docker-compose.yml).
Como não quero vazar as minhas chaves da Honeycomb, a aplicação lida com a ausência delas simplesmente pulando a configuração do tracing.

No geral, é um código Rust padrão, sem muita firula.
A escolha mais exótica foi como lidei com o requerimento de algumas strings terem limitação de tamanho. 
Para isso, defini o tipo `LengthRestrictedString` (que está no módulo `utils`), que usa [const generics](https://practice.rs/generics-traits/const-generics.html) para encodar os limites de tamanho na string no sistema de tipos.
Isso é interessante por ser uma funcionalidade recente, mas muito poderosa do Rust.
Foi preciso derivar manualmente o trait `Deserialize` para o serde entender que a string tem essa limitação de tamanho.
Mas, fazendo isso, foi possível usar esse tipo dentro do tipo que recebe os dados enviados pelo usuário e o serde e o actix-web conseguem fazer as validações, sem eu precisar escrever uma camada explicitamente.

## Load Balancer

Como especificado nas regras, eu usei o Nginx.
Brinquei um pouco com algumas configurações de otimização para ele, como aumentar o número de `worker_connections` e aumentar o número de files descriptors disponíveis mas, até onde testei, isso não teve impacto significativo.
Então, mantive a configuração praticamente mínima.

## Banco de dados

Usei o bom e velho Postgres.
Implementar a maior parte dos requerimentos da rinha nele é trivial.
Como a relação entre pessoas e stacks é 1-N, a forma normal/portável de se implementar isso em um banco relacional seria criando uma tabela separada para as stacks, com uma foreign key apontando para a tabela de pessoas.
Mas o Postgres oferece outras funcionalidades que permitem implementar isso em uma tabela só, com uma coluna `jsonb` ou `array`.
Optei por implementar como array.
Baseado em algumas coisas que li, parece ser a escolha que oferece maior performance.
Mas, como não medi isso, é só uma suspeita e não algo que me sinto confortável em afirmar.

O que causou a maior complexidade (para mim e suspeito que também para a maioria dos competidores) foi o requerimento de ser possível fazer buscas por substrings.
Como os requerimentos não são 100% explícitos sobre isso, interpretei que o match só deveria acontecer para substrings exatas, a menos de casing, o que implementei fazer um `SELECT` filtrando por `ILIKE`.
A dificuldade é que, sem índices, os operadores `LIKE/ILIKE` são _extremamente_ lentos, dado que não só exigem um fullscan da tabela mas também fazer um scan linear em cada valor, já que o pattern que estamos buscando não é left-anchored.
Isso foi resolvido introduzido um índice de trigram, do tipo GIST.
Encontrei [este artigo](https://alexklibisz.com/2022/02/18/optimizing-postgres-trigram-search) que dá bons detalhes sobre como isso funciona.
Como a busca tem que retornar matches no nome, sobrenome ou nas stacks, criei uma coluna computada combinada todos os valores e apliquei o índice sobre ela.
Teoricamente deveria ser possível mover a computação só para o índice mas, fazendo dessa forma, não consegui convencer o Postgres a usar o índice.
Eventualmente quero voltar para essa questão, dado que essa escolha entre a coluna computada e o índice complexo traz tradeoffs de performance: a primeira é mais eficiente em situações read-heavy, enquanto a segunda é favorável em situações write-heavy.

A última sutileza é que precisei definir uma função para fazer o join da array de stacks, porque a `array_to_text` do Postgres não tem a volatilidade correta para ser utiliza em uma coluna computada.

## Cache

Não tem :)
