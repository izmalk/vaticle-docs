---
pageTitle: Connections
keywords: typedb, basics, connect, connection, session, database
longTailKeywords: basic concepts of typedb, typedb connection, typedb database, typedb session
summary: Brief description of connection to TypeDB.
toc: false
---

# Connections

## Clients

TypeDB server accepts remote connections from a number of [TypeDB clients](../../02-clients/00-clients.md) via 
[gRPC](https://en.wikipedia.org/wiki/GRPC) protocol.

Once connected, TypeDB clients can manage [databases](#databases) and [sessions](#sessions).

<div class="note">
[Note]
It’s recommended to instantiate a single client per application.
</div>

## Databases

A database is the outermost container for data in a TypeDB. Like a relational database, it is commonly
known to be a good practice to create a single database per application, but it is absolutely fine to create as many
databases as your application needs. As a rule of thumb, it is recommended to start off with one database and create
more if the requirement arises.

<div class="note">
[Note]
TypeDB optimised for lesser number of databases. So the best practise would be to keep no more than 10 databases on 
TypeDB server.
</div>

Databases are isolated from one another. Even when running on the same TypeDB Server, it is not possible to connect to
one database but perform operations on another one. But you can connect to multiple databases simultaneously.

A database is made of [schema](02-schema.md), and [data](05-read.md).

## Sessions

A session holds a connection to a particular database. This connection then allows opening 
[transactions](#transactions) to carry out [queries](04-write.md).

There are two types of session:

- `Schema` — addresses only schema of a database (types and rules).
- `Data` — addresses only data of a database (instances of types in a schema).

A session must explicitly state whether it addresses Schema or Data of a database.

### Best Practices

Because of intermittent network failures, it is recommended to keep sessions relatively short-lived.

A good principle is that sessions group logically coherent transactions. For example, when loading a web page, one
session should be used to open one or more transactions to load the page data.

## Transactions


## ACID guarantees

