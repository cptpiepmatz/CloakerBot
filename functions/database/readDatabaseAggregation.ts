import {AggregateOptions, Document} from "mongodb";
import DatabaseTables from "../../enums/DatabaseTables";

/**
 * Reads data from the database using an aggregation pipeline.
 * @param collectionName The name of the collection
 * @param pipeline Array containing all the aggregation framework command for
 *                 the execution
 * @param options Optional settings
 */
async function readDatabaseAggregation(
    collectionName: DatabaseTables,
    pipeline: Document[],
    options?: AggregateOptions
): Promise<Array<object>> {
  const collection = dbo.collection(collectionName);
  let cursor = collection.aggregate(pipeline, options);
  return await cursor.toArray();
}

export default readDatabaseAggregation;
