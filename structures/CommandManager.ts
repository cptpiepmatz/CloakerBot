import helpCommand from "../commands/help";
import randomCommand from "../commands/random";
import ranksCommand from "../commands/ranks";
import takedownCommand from "../commands/takedown";
import takedownRoleCommand from "../commands/takedownRole";
import {ApplicationCommand, Guild, GuildApplicationCommandPermissionData} from "discord.js";
import CommandFunction from "../types/CommandFunction";

export default class CommandManager {

  private _registeredCommands: Record<string, ApplicationCommand> = {};
  get registeredCommands() {return this._registeredCommands};

  private _commandFunctions: Record<string, CommandFunction> = {};
  get commandFunctions() {return this._commandFunctions};

  constructor() {
    for (let commandFunction of [
      helpCommand,
      randomCommand,
      ranksCommand,
      takedownCommand,
      takedownRoleCommand
    ]) {
      this._commandFunctions[commandFunction.commandData.name] = commandFunction;
    }
  }

  async fetchCommands(): Promise<void> {
    const registeredCommands = await client.application!.commands.fetch();
    for (let registeredCommand of registeredCommands.values()) {
      this._registeredCommands[registeredCommand.name] = registeredCommand;
    }
  }

  async updatePermissions(guild: Guild): Promise<void> {
    const fullPermissions: GuildApplicationCommandPermissionData[] = [];
    for (let [name, command] of Object.entries(this.registeredCommands)) {
      if (command.defaultPermission) continue;
      const permissions = await this.commandFunctions[name].getPermissions!(guild);
      fullPermissions.push({
        id: command.id,
        permissions: permissions
      });
    }
    await guild.commands.permissions.set({fullPermissions: fullPermissions});
  }

}
