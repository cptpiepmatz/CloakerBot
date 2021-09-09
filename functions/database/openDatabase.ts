import {MongoClient} from "mongodb";

/**
 * Opens a mongodb connection.
 * <p>Applies the connection to the global scope.
 */
async function openDatabase(): Promise<void> {
  const uri = "mongodb://localhost:27017";
  const mongoClient = new MongoClient(uri);

  try {
    await mongoClient.connect();

    if (process.argv.slice(2).includes("dev")) {
      globalThis.dbo = mongoClient.db("cloaker_bot_dev");
    }
    else {
      globalThis.dbo = mongoClient.db("cloaker_bot");
    }
  }
  catch (e) {
    console.error(e);
    process.exit(1);
  }
}

export default openDatabase;
