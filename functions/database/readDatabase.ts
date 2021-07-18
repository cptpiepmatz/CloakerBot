import {Guild, GuildResolvable} from "discord.js";
import {Document} from "mongodb";
import DatabaseTables from "../../enums/DatabaseTables";

/**
 * Reads data from the database for a specific or all guilds.
 * @param collectionName The collection to read from
 * @param guild The guild to read for or "all"
 * @param query The query to with "all"
 */
async function readDatabase(
    collectionName: DatabaseTables,
    guild: GuildResolvable | "all",
    query?: Document
): Promise<object | Array<object> | undefined> {
  const collection = globalThis.dbo.collection(collectionName);

  if (guild !== "all") {
    let guildId = globalThis.client.guilds.resolve(guild);
    return await collection.findOne({GuildID: guildId})?.toArray();
  }

  let cursor = collection.find(query);
  let returnData = await cursor.toArray();
  cursor.close();
  return returnData;
}

export default readDatabase;
