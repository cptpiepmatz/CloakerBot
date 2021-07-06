"use strict";

const { MongoClient } = require("mongodb");

/**
 * Opens a mongodb connection.
 * <p>Applies the connection to global.dbo
 *
 * @returns {Promise<void>}
 */
async function openDatabase() {
  const uri = "mongodb://localhost:27017";
  const mongoClient = new MongoClient(uri, {
    useUnifiedTopology: true,
    useNewUrlParser: true
  });

  try {
    await mongoClient.connect();

    global.dbo = mongoClient.db("cloaker_bot");
  }
  catch (e) {
    console.error(e);
  }
}

module.exports = openDatabase;