"use strict";

const Discord = require("discord.js");

/**
 * Reads data from the db using an aggregation pipeline.
 *
 * @param {string} collectionName The name of the collection
 * @param {array} pipeline Array containing all the aggregation framework
 *                         commands for the execution
 * @param {object} [options] Optional settings
 * @returns {Promise<object[]>}
 */
async function readDatabaseAggregation(collectionName, pipeline, options) {
  const collection = dbo.collection(collectionName);
  let cursor = collection.aggregate(pipeline, options);
  return await cursor.toArray();
}

module.exports = readDatabaseAggregation;
