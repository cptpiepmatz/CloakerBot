import DatabaseTables from "../../enums/DatabaseTables";
import {GuildResolvable} from "discord.js";
import ResolvedNullError from "../../error/ResolvedNullError";

/**
 * Writes data to the database.
 * @param collectionName The collection to store the data
 * @param guild The guild is data is stored for
 * @param data The data to store
 */
async function writeDatabase(
    collectionName: DatabaseTables,
    guild: GuildResolvable,
    data: object
): Promise<void> {
  const collection = dbo.collection(collectionName);
  const resolvedGuild = client.guilds.resolve(guild);
  if (!resolvedGuild) throw new ResolvedNullError(guild, client.guilds);

  let insertData = {
    $set: {
      GuildName: resolvedGuild.name
    }
  };

  let filter = {
    GuildID: resolvedGuild.id
  };

  for (let dataKey in data) {
    if (data.hasOwnProperty(dataKey)) {
      // @ts-ignore I really don't care
      insertData.$set[dataKey] = data[dataKey];
    }
  }

  // uses the updateOne to update if the data is already in the database
  // adds the data if not existing
  await collection.updateOne(filter, insertData, {upsert: true});
}

export default writeDatabase;
