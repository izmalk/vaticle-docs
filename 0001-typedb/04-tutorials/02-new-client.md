---
pageTitle: How to create a new TypeDB Client
keywords: typedb, client, driver, grpc
longTailKeywords: TypeDB client, building new client, developing new driver
Summary: Tutorial on how to build a new TypeDB Client.
---

# How to create a new TypeDB Client

This tutorial can guide us through the very beginning of creating a new TypeDB Client. We strongly recommend 
using one of the existing TypeDB Clients first to gain some experience with the TypeDB. See the list of available 
Clients on the [Clients overview](../../02-clients/00-clients.md) page.

There are many places we could start building a TypeDB Client. 
In this tutorial, we start by attempting to make a single gRPC call to a TypeDB server to create a database.

## Step 1: Create the main function to connect to a server

Create a `TypeDB` source file in the root of the project, which should expose a function named `coreClient`, 
taking `address` as a parameter.

<!-- #todo add the imports! --->

<div class="note">
[Important]
Import statements are not included in this tutorial, except when importing from external libraries such as the 
TypeDB protobuf definitions.
</div>

<div class="tabs dark">

[tab:Java]
<!-- test-ignore -->
```java
// TypeDB.java
public class TypeDB {
    public static TypeDBClient coreClient(String address) {
        return new CoreClient(address);
    }
}
```
[tab:end]

[tab:Node.js]
```typescript
// TypeDB.ts
export namespace TypeDB {
    export function coreClient(address: string): TypeDBClient {
        return new CoreClient(address);
    }
}
```
[tab:end]

[tab:Python]
```python
# typedb/client.py (named to allow importing from typedb.client)
class TypeDB:
    @staticmethod
    def core_client(address: str, parallelisation: int = 2) -&gt; TypeDBClient:
        return _CoreClient(address, parallelisation)
```
[tab:end]

</div>

## Step 2: Database manager

`TypeDBClient` is not yet defined. Create a new directory named `api/connection` and create a `TypeDBClient` file there:

<div class="note">
[Note]
if your language doesn't have interfaces or abstract classes, make `TypeDB.coreClient` return `CoreClient` instead, 
and skip this step.
</div>

<div class="tabs dark">

[tab:Java]
<!-- test-ignore -->
```java
// api/connection/TypeDBClient.java
public interface TypeDBClient extends AutoCloseable {
    DatabaseManager databases();
    void close();
}
```
[tab:end]

[tab:Node.js]
```typescript
// api/connection/TypeDBClient.ts
export interface TypeDBClient {
    readonly databases: DatabaseManager;
    close(): Promise&lt;void&gt;;
}
```
[tab:end]

[tab:Python]
```python
# typedb/api/connection/client.py
from abc import ABC, abstractmethod

class TypeDBClient(ABC):
    @abstractmethod
    def databases(self) -&gt; DatabaseManager:
        pass

    @abstractmethod
    def close(self) -&gt; None:
        pass

    @abstractmethod
    def __enter__(self):
        pass

    @abstractmethod
    def __exit__(self, exc_type, exc_val, exc_tb):
        pass
```
[tab:end]

</div>

## Step 3: gRPC connection implementation

The next step is to implement `connection/TypeDBClient` and its subclass `connection/core/CoreClient`.
Create the directory structure: `connection/core` in the root of your project.

Name the classes depending on language conventions: in Java/TypeScript, `TypeDBClientImpl` and `CoreClient`; in Python, 
`_TypeDBClient` and `_CoreClient`.

Ensure that you've imported gRPC into your project, and refer to the [gRPC docs](https://grpc.io/docs/languages/) to 
learn how to create a Channel - the code varies by language.

<div class="note">
[Note]
In languages with no inheritance, adhere to this project structure as closely as possible, perhaps by writing 
top-level functions in the respective locations.
</div>

<div class="tabs dark">

[tab:Java]
<!-- test-ignore -->
```java
// connection/TypeDBClientImpl.java
public abstract class TypeDBClientImpl implements TypeDBClient {
    private final TypeDBDatabaseManagerImpl databaseMgr;

    protected TypeDBClientImpl() {
        databaseMgr = new TypeDBDatabaseManagerImpl(this);
    }

    @Override
    public TypeDBDatabaseManagerImpl databases() {
        return databaseMgr;
    }

    public abstract ManagedChannel channel();

    public abstract TypeDBStub stub();

    @Override
    public void close() {
        try {
            channel().shutdown().awaitTermination(10, TimeUnit.SECONDS);
        } catch (InterruptedException e) {
            Thread.currentThread().interrupt();
        }
    }
}

// connection/core/CoreClient.java
public class CoreClient extends TypeDBClientImpl {
    private final ManagedChannel channel;
    private final TypeDBStub stub;

    public CoreClient(String address) {
        super();
        channel = NettyChannelBuilder.forTarget(address).usePlaintext().build();
        stub = CoreStub.create(channel);
    }

    @Override
    public ManagedChannel channel() {
        return channel;
    }

    @Override
    public TypeDBStub stub() {
        return stub;
    }
}
```
[tab:end]

[tab:Node.js]
```typescript
// connection/TypeDBClientImpl.ts
export abstract class TypeDBClientImpl implements TypeDBClient {
    private _isOpen: boolean;

    protected constructor() {
        this._isOpen = true;
    }

    isOpen(): boolean {
        return this._isOpen;
    }

    abstract get databases(): TypeDBDatabaseManagerImpl;

    abstract stub(): TypeDBStub;

    async close(): Promise&lt;void&gt; {
        if (this._isOpen) {
            this._isOpen = false;
        }
    }
}

// connection/core/CoreClient.ts
export class CoreClient extends TypeDBClientImpl {
    private readonly _stub: CoreStub;
    private readonly _databases: TypeDBDatabaseManagerImpl;

    constructor(address: string) {
        super();
        this._stub = new CoreStub(address);
        this._databases = new TypeDBDatabaseManagerImpl(this._stub);
    }

    get databases(): TypeDBDatabaseManagerImpl {
        return this._databases;
    }

    stub(): TypeDBStub {
        return this._stub;
    }

    async close(): Promise&lt;void&gt; {
        await super.close();
        this._stub.close();
    }
}
```
[tab:end]

[tab:Python]
```python
# typedb/connection/client.py
class _TypeDBClientImpl(TypeDBClient):
    def __init__(self):
        pass

    def databases(self) -&gt; _TypeDBDatabaseManagerImpl:
        pass

    def stub(self) -&gt; TypeDBStub:
        pass

    def __enter__(self):
        return self

    def __exit__(self, exc_type, exc_val, exc_tb):
        self.close()
        if exc_tb is not None:
            return False

    def close(self) -&gt; None:
        pass

# typedb/connection/core/client.py
from grpc import Channel, insecure_channel

class _CoreClient(_TypeDBClientImpl):
    def __init__(self, address: str):
        super(_CoreClient, self).__init__()
        self._channel = insecure_channel(address)
        self._stub = _CoreStub(self._channel)
        self._databases = _TypeDBDatabaseManagerImpl(self._stub)

    def databases(self) -&gt; _TypeDBDatabaseManagerImpl:
        return self._databases

    def stub(self) -&gt; _CoreStub:
        return self._stub

    def close(self) -&gt; None:
        super().close()
        self._channel.close()
```
[tab:end]

</div>

## Step 4: Implementing database creation

Finally, we implement `DatabaseManager`, and `CoreStub` to set up gRPC calls to the server.

<div class="note">
[Important]
You'll need to compile TypeDB's [protocol](https://github.com/vaticle/typedb-protocol) in order to do this.
Most languages have protobuf compilers that you can use to generate a TypeDB protocol library for your language.
</div>

<div class="tabs dark">

[tab:Java]
<!-- test-ignore -->
```java
// api/database/DatabaseManager.java
public interface DatabaseManager {
    void create(String name);
}

// connection/TypeDBDatabaseManagerImpl.java
import com.vaticle.typedb.protocol.CoreDatabaseProto;

public class TypeDBDatabaseManagerImpl implements DatabaseManager {
    private final TypeDBClientImpl client;

    public TypeDBDatabaseManagerImpl(TypeDBClientImpl client) {
        this.client = client;
    }

    @Override
    public void create(String name) {
        stub().databasesCreate(CoreDatabaseProto.CoreDatabaseManager.Create.Req.newBuilder().setName(name).build());
    }

    TypeDBStub stub() {
        return client.stub();
    }
}

// common/rpc/TypeDBStub.java
import com.vaticle.typedb.protocol.CoreDatabaseProto;
import com.vaticle.typedb.protocol.TypeDBGrpc;

public abstract class TypeDBStub {
    public CoreDatabaseProto.CoreDatabaseManager.Create.Res databasesCreate(CoreDatabaseProto.CoreDatabaseManager.Create.Req request) {
        return blockingStub().databasesCreate(request);
    }

    protected abstract TypeDBGrpc.TypeDBBlockingStub blockingStub();
}

// connection/core/CoreStub.java
import com.vaticle.typedb.protocol.TypeDBGrpc;
import io.grpc.ManagedChannel;

public class CoreStub extends TypeDBStub {
    private final ManagedChannel channel;
    private final TypeDBGrpc.TypeDBBlockingStub blockingStub;

    private CoreStub(ManagedChannel channel) {
        super();
        this.channel = channel;
        this.blockingStub = TypeDBGrpc.newBlockingStub(channel);
    }

    public static CoreStub create(ManagedChannel channel) {
        return new CoreStub(channel);
    }

    @Override
    protected TypeDBGrpc.TypeDBBlockingStub blockingStub() {
        return blockingStub;
    }
}
```
[tab:end]

[tab:Node.js]
```typescript
// api/connection/database/TypeDBClient.ts
export interface DatabaseManager {
    create(name: string): Promise&lt;void&gt;;
}

// connection/TypeDBDatabaseManagerImpl.ts
import { CoreDatabaseManager } from "typedb-protocol/core/core_database_pb";

export class TypeDBDatabaseManagerImpl implements DatabaseManager {
    private readonly _stub: TypeDBStub;

    constructor(client: TypeDBStub) {
        this._stub = client;
    }

    public create(name: string): Promise&lt;void&gt; {
        return this._stub.databasesCreate(new CoreDatabaseManager.Create.Req().setName(name));
    }

    stub() {
        return this._stub;
    }
}

// common/rpc/TypeDBStub.ts
import { CoreDatabaseManager } from "typedb-protocol/core/core_database_pb";
import { TypeDBClient } from "typedb-protocol/core/core_service_grpc_pb";

export abstract class TypeDBStub {
    databasesCreate(req: CoreDatabaseManager.Create.Req): Promise&lt;void&gt; {
        return new Promise((resolve, reject) =&gt; {
            this.stub().databases_create(req, (err) =&gt; {
                if (err) reject(new Error(err));
                else resolve();
            })
        });
    }

    abstract stub(): TypeDBClient;
}

// connection/core/CoreStub.ts
import { ChannelCredentials } from "@grpc/grpc-js";
import { TypeDBClient } from "typedb-protocol/core/core_service_grpc_pb";

export class CoreStub extends TypeDBStub {
    private readonly _stub: TypeDBClient;

    constructor(address: string) {
        super();
        this._stub = new TypeDBClient(address, ChannelCredentials.createInsecure());
    }

    stub(): TypeDBClient {
        return this._stub;
    }

    close(): void {
        this._stub.close();
    }
}
```
[tab:end]

[tab:Python]
```python
# typedb/api/connection/database.py
from abc import ABC, abstractmethod

class DatabaseManager(ABC):
    @abstractmethod
    def create(self, name: str) -&gt; None:
        pass

# typedb/connection/database_manager.py
import typedb_protocol.core.core_database_pb2 as core_database_proto

class _TypeDBDatabaseManagerImpl(DatabaseManager):
    def __init__(self, stub: TypeDBStub):
        self._stub = stub

    def create(self, name: str) -&gt; None:
        req = core_database_proto.CoreDatabaseManager.Create.Req()
        req.name = name
        self._stub.databases_create(req)

    def stub(self) -&gt; TypeDBStub:
        return self._stub

# typedb/common/rpc/stub.py
import typedb_protocol.core.core_database_pb2 as core_database_proto
import typedb_protocol.core.core_service_pb2_grpc as core_service_proto

class TypeDBStub(ABC):
    def databases_create(self, req: core_database_proto.CoreDatabaseManager.Create.Req) -&gt; core_database_proto.CoreDatabaseManager.Create.Res:
        return self.stub().databases_create(req)

    def stub(self) -&gt; core_service_proto.TypeDBStub:
        pass

# typedb/connection/core/stub.py
from grpc import Channel
import typedb_protocol.core.core_service_pb2_grpc as core_service_proto

class _CoreStub(TypeDBStub):
    def __init__(self, channel: Channel):
        super(_CoreStub, self).__init__()
        self._channel = channel
        self._stub = core_service_proto.TypeDBStub(channel)

    def stub(self) -&gt; TypeDBStub:
        return self._stub
```
[tab:end]

</div>

## Step 5: Testing

At this point, we have all the necessary components to create a database! 
[Run the TypeDB server locally](../01-start/02-installation.md) and 
create a test function:

<div class="tabs dark">

[tab:Java]
<!-- test-ignore -->
```java
public static void typeDBClientTest() {
    try (TypeDBClient client = TypeDB.coreClient("127.0.0.1:1729")) {
        client.databases().create("typedb");
    }
}
```
[tab:end]

[tab:Node.js]
```typescript
async function typeDBClientTest() {
    try {
        const client = TypeDB.coreClient("127.0.0.1:1729");
        await client.databases().create("typedb");
    } finally {
        client?.close();
    }
}
```
[tab:end]

[tab:Python]
```python
def typedb_client_test():
    with TypeDB.core_client("127.0.0.1:1729") as client:
        client.databases().create("typedb")
```
[tab:end]

</div>

Run the test function.

Now we can verify that the database was created successfully using 
[TypeDB Console](../../02-clients/02-console.md#database-management-commands) `database list` command, or 
by running the test again (which will throw an error saying that the database already exists).

That concludes the basics tutorial for creating a new TypeDB Client.

Refer to the [Developing a new TypeDB Client](../../02-clients/07-new-client.md) page for more information of the 
remaining components needed to open transactions, run queries, and take the client to 100% completion.

We recommend using one of our existing Clients as a reference, and copying the implementation into your chosen language.
