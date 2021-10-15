import {GuildResolvable} from "discord.js";
import Logger from "js-logger";

/**
 * Removes all database entries for a guild.
 * @param guild Guild to purge from the database
 */
async function purgeDatabase(guild: GuildResolvable): Promise<void> {
  let collections = await dbo.collections();
  let resolvedGuild = client.guilds.resolve(guild);
  if (!resolvedGuild) {
    Logger.error("couldn't resolve guild.");
    Logger.warn("will try to use as guild id this: " + guild);
    for (let collection of collections) {
      await collection.deleteOne({GuildID: guild});
    }
    Logger.info("wiped database entries with " + guild);
    return;
  }
  for (let collection of collections) {
    await collection.deleteOne({GuildID: resolvedGuild.id});
  }
  Logger.info("Wiped database entries for " + resolvedGuild.name);
}

export default purgeDatabase;
