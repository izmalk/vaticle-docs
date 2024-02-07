const { TypeDB } = require("typedb-driver/TypeDB");
const { SessionType } = require("typedb-driver/api/connection/TypeDBSession");
const { TransactionType } = require("typedb-driver/api/connection/TypeDBTransaction");
const { TypeDBOptions } = require("typedb-driver/api/connection/TypeDBOptions");

async function main() {
    const DB_NAME = "iam";
    const SERVER_ADDR = "127.0.0.1:1729";

    console.log("TypeDB Manual sample code");

    const driver = await TypeDB.coreDriver(SERVER_ADDR);

    let dbs = await driver.databases.all();
    for(let i = 0; i < dbs.length; i++) {
        console.log("DB " + dbs[i]);
    }

    if (await driver.databases.contains(DB_NAME)) {
        let x = await driver.databases.get(DB_NAME);
        await x.delete();
    }
    await driver.databases.create(DB_NAME);

    if (driver.databases.contains(DB_NAME)) {
        console.log("Database setup complete.");
    }

    try {
        session = await driver.session(DB_NAME, SessionType.SCHEMA);
        try {
            transaction = await session.transaction(TransactionType.WRITE);
            const define_query = `
                                define
                                email sub attribute, value string;
                                name sub attribute, value string;
                                friendship sub relation, relates friend;
                                user sub entity,
                                    owns email @key,
                                    owns name,
                                    plays friendship:friend;
                                admin sub user;
                                `;
            await transaction.query.define(define_query);
            await transaction.commit();
        }
        finally {if (transaction.isOpen()) {await transaction.close()};}
    }
    finally {await session?.close();}

    try {
        session = await driver.session(DB_NAME, SessionType.SCHEMA);
        try {
            transaction = await session.transaction(TransactionType.WRITE);
            const undefine_query = "undefine admin sub user;";
            await transaction.query.undefine(undefine_query);
            await transaction.commit();
        }
        finally {if (transaction.isOpen()) {await transaction.close()};}
    }
    finally {await session?.close();}

    try {
        session = await driver.session(DB_NAME, SessionType.DATA);
        try {
            transaction = await session.transaction(TransactionType.WRITE);
            const insert_query = `
                                insert
                                $user1 isa user, has name "Alice", has email "alice@vaticle.com";
                                $user2 isa user, has name "Bob", has email "bob@vaticle.com";
                                $friendship (friend:$user1, friend: $user2) isa friendship;
                                `;
            await transaction.query.insert(insert_query);
            await transaction.commit();
        }
        finally {if (transaction.isOpen()) {await transaction.close()};}
    }
    finally {await session?.close();}

    try {
        session = await driver.session(DB_NAME, SessionType.DATA);
        try {
            transaction = await session.transaction(TransactionType.WRITE);
            const match_insert_query = `
                                match
                                $u isa user, has name "Bob";
                                insert
                                $new-u isa user, has name "Charlie", has email "charlie@vaticle.com";
                                $f($u,$new-u) isa friendship;
                                `;
            let response = await transaction.query.insert(match_insert_query).collect();
            if (response.length == 1) {
                await transaction.commit();
            }
            else {await transaction.close();}
        }
        finally {if (transaction.isOpen()) {await transaction.close()};}
    }
    finally {await session?.close();}

    try {
        session = await driver.session(DB_NAME, SessionType.DATA);
        try {
            transaction = await session.transaction(TransactionType.WRITE);
            const delete_query = `
                                match
                                $u isa user, has name "Charlie";
                                $f ($u) isa friendship;
                                delete
                                $f isa friendship;
                                `;
            await transaction.query.delete(delete_query);
            await transaction.commit();
        }
        finally {if (transaction.isOpen()) {await transaction.close()};}
    }
    finally {await session?.close();}

    try {
        session = await driver.session(DB_NAME, SessionType.DATA);
        try {
            transaction = await session.transaction(TransactionType.WRITE);
            const update_query = `
                                match
                                $u isa user, has name "Charlie", has email $e;
                                delete
                                $u has $e;
                                insert
                                $u has email "charles@vaticle.com";
                                `;
            let response = await transaction.query.update(update_query).collect();
            if (response.length == 1) {
                await transaction.commit();
            }
            else {await transaction.close();}
        }
        finally {if (transaction.isOpen()) {await transaction.close()};}
    }
    finally {await session?.close();}

    try {
        session = await driver.session(DB_NAME, SessionType.DATA);
        try {
            transaction = await session.transaction(TransactionType.READ);
            const fetch_query = `
                                match
                                $u isa user;
                                fetch
                                $u: name, email;
                                `;
            let response = await transaction.query.fetch(fetch_query).collect();
            k = 0;
            for(let i = 0; i < response.length; i++) {
                k++;
                console.log("User #" + k + ": " + JSON.stringify(response[i], null, 4));
            }
        }
        finally {if (transaction.isOpen()) {await transaction.close()};}
    }
    finally {await session?.close();}

    try {
        session = await driver.session(DB_NAME, SessionType.DATA);
        try {
            transaction = await session.transaction(TransactionType.READ);
            const get_query = `
                                match
                                $u isa user, has email $e;
                                get
                                $e;
                                `;
            let response = await transaction.query.get(get_query).collect();
            k = 0;
            for(let i = 0; i < response.length; i++) {
                k++;
                console.log("Email #" + k + ": " + response[i].get("e").value);
            }
        }
        finally {if (transaction.isOpen()) {await transaction.close()};}
    }
    finally {await session?.close();}

    try {
        session = await driver.session(DB_NAME, SessionType.SCHEMA);
        try {
            transaction = await session.transaction(TransactionType.WRITE);
            const define_query = `
                                define
                                rule users:
                                when {
                                    $u isa user;
                                } then {
                                    $u has name "User";
                                };
                                `;
            await transaction.query.define(define_query);
            await transaction.commit();
        }
        finally {if (transaction.isOpen()) {await transaction.close()};}
    }
    finally {await session?.close();}

    try {
        session = await driver.session(DB_NAME, SessionType.DATA);
        try {
            let options = new TypeDBOptions();
            options.infer = true;
            transaction = await session.transaction(TransactionType.READ, options);
            const fetch_query = `
                                match
                                $u isa user;
                                fetch
                                $u: name, email;
                                `;
            let response = await transaction.query.fetch(fetch_query).collect();
            k = 0;
            for(let i = 0; i < response.length; i++) {
                k++;
                console.log("User #" + k + ": " + JSON.stringify(response[i], null, 4));
            }
        }
        finally {if (transaction.isOpen()) {await transaction.close()};}
    }
    finally {await session?.close();}
};

main();
