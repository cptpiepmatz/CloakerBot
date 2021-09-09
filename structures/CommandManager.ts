import helpCommand from "../commands/help";
import randomCommand from "../commands/random";
import ranksCommand from "../commands/ranks";
import takedownCommand from "../commands/takedown";
import takedownRoleCommand from "../commands/takedownRole";
import dangerZoneCommand from "../commands/dangerZone";
import {
  ApplicationCommand,
  ApplicationCommandPermissionData,
  Guild,
  GuildApplicationCommandPermissionData,
  Message,
  Role,
  Snowflake
} from "discord.js";
import CommandFunction from "../types/CommandFunction";
import Logger from "js-logger";

export default class CommandManager {

  private _registeredCommands: Record<string, ApplicationCommand> = {};
  get registeredCommands() {return this._registeredCommands};

  private _commandFunctions: Record<string, CommandFunction> = {};
  get commandFunctions() {return this._commandFunctions};

  private initDone: Record<Snowflake, boolean> = {};

  constructor() {
    for (let commandFunction of [
      helpCommand,
      randomCommand,
      ranksCommand,
      takedownCommand,
      takedownRoleCommand,
      dangerZoneCommand
    ]) {
      Logger.trace("constructing command manager");
      this._commandFunctions[commandFunction.commandData.name] = commandFunction;
    }
  }

  async fetchCommands(): Promise<void> {
    Logger.debug("fetching commands");
    const registeredCommands = await client.application!.commands.fetch();
    for (let registeredCommand of registeredCommands.values()) {
      Logger.debug("fetched command:");
      Logger.debug(registeredCommand);
      this._registeredCommands[registeredCommand.name] = registeredCommand;
    }
  }

  async updatePermissions(updateFor: Guild | Role, firstIsDummy?: Guild | Role): Promise<void> {
    let guild: Guild;
    if (firstIsDummy) {
      if (firstIsDummy instanceof Guild) guild = firstIsDummy;
      else guild = firstIsDummy.guild;
    }
    else if (updateFor instanceof Guild) guild = updateFor;
    else guild = updateFor.guild;

    Logger.info("updating permissions for " + guild.name + "<" +
      guild.id + ">");

    const fullPermissions: GuildApplicationCommandPermissionData[] = [];
    for (let [name, command] of Object.entries(this.registeredCommands)) {
      if (command.defaultPermission) continue;
      const permissions = await this.commandFunctions[name].getPermissions!(guild);
      fullPermissions.push({
        id: command.id,
        permissions: permissions.slice(0, 10)
      });
      if (permissions.length > 10) {
        Logger.warn(`command '${name}' has too many valid users`);
        Logger.debug(permissions);
      }
    }
    Logger.debug("set permissions:");
    Logger.debug(fullPermissions);
    await guild.commands.permissions.set({fullPermissions: fullPermissions});
  }

  async initPermissions(message: Message): Promise<void> {
    if (!message.guild) return;
    const guild = message.guild;
    if (this.initDone[guild.id]) return;
    await this.updatePermissions(guild);
    this.initDone[guild.id] = true;
    Logger.info("initialized permissions for " + guild.name + "<" +
      guild.id + ">");
  }

}
