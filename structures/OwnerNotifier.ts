import readDatabase from "../functions/database/readDatabase";
import DatabaseTables from "../enums/DatabaseTables";
import OwnerNotificationsDBData from "../types/database/OwnerNotificationsDBData";
import {Interaction, Message, MessageActionRow, MessageButton, Snowflake} from "discord.js";

import update200 from "../owner_messages/Update 2.0.0";
import AwaitLock from "await-lock";
import Logger from "js-logger";
import memberString from "../functions/util/memberString";

const ownerMessages = [update200];

export default class OwnerNotifier {

  private ownerMessageStates: Record<Snowflake, {
    id: Snowflake;
    username: string;
    discriminator: string;
    latestMessageIndex: number;
    allowDMs?: boolean;
  }> = {};

  private lock = new AwaitLock();

  private constructor(dbData: OwnerNotificationsDBData[]) {
    Logger.trace("constructing owner notifier");
    for (let date of dbData) {
      Logger.trace(date);
      this.ownerMessageStates[date.UserID] = {
        id: date.UserID,
        username: date.UserName,
        discriminator: date.UserDiscriminator,
        allowDMs: date.allowDMs,
        latestMessageIndex: date.latestMessageIndex
      };
    }
  }

  static async create(): Promise<OwnerNotifier> {
    Logger.trace("creating owner notifier");
    const dbData = await readDatabase(DatabaseTables.OWNER_NOTIFICATIONS, "all");
    return new OwnerNotifier(dbData as OwnerNotificationsDBData[]);
  }

  async messageOwner(message: Message): Promise<void> {
    if (!message.guild) return;
    const owner = await message.guild.fetchOwner();
    Logger.debug("fetched owner: " + memberString(owner));
    if (this.ownerMessageStates[owner.id]?.allowDMs ?? true) {
      Logger.debug("will message owner");
      await this.lock.acquireAsync();
      try {
        Logger.debug("got lock");
        const currentStates = this.ownerMessageStates[owner.id] ?? {
          id: owner.id
        };
        const latestMessageIndex = currentStates.latestMessageIndex ?? -1;
        if (latestMessageIndex >= ownerMessages.length - 1) return;
        Logger.info("messaging owner " + memberString(owner));
        for (
          let i = latestMessageIndex + 1;
          i < ownerMessages.length;
          i++
        ) {
          await owner.send(ownerMessages[i]);
        }
        currentStates.latestMessageIndex = ownerMessages.length - 1;
        currentStates.username = owner.user.username;
        currentStates.discriminator = owner.user.discriminator;
        this.ownerMessageStates[owner.id] = currentStates;

        await this.updateDB(owner.id);
      }
      finally {
        this.lock.release();
      }
    }
  }

  async unsubscribe(interaction: Interaction) {
    if (!interaction.isButton()) return;
    if (interaction.customId !== "dm-unsub") return;

    Logger.info(interaction.user.username + "#" +
      interaction.user.discriminator + " unsubscribed from owner notifications");

    await interaction.deferReply();

    const userId = interaction.user.id;
    this.ownerMessageStates[userId] = {
      id: userId,
      username: interaction.user.username,
      discriminator: interaction.user.discriminator,
      latestMessageIndex: this.ownerMessageStates[userId].latestMessageIndex ?? 0,
      allowDMs: false
    }

    await this.updateDB(userId);

    const dmChannel = await interaction.user.createDM();
    const message = await dmChannel.messages.fetch(interaction.message.id)
    message?.edit({components: []});

    await interaction.editReply({
      content: "Unsubscribed successfully"
    });

    await interaction.user.send({
      content: "You're still able to opt in back again.",
      components: [new MessageActionRow().addComponents(new MessageButton({
        customId: "dm-sub",
        label: "Subscribe again",
        style: "PRIMARY"
      }))]
    });
  }

  async subscribe(interaction: Interaction) {
    if (!interaction.isButton()) return;
    if (interaction.customId !== "dm-sub") return;

    Logger.info(interaction.user.username + "#" +
      interaction.user.discriminator + " subscribed from owner notifications");

    await interaction.deferReply();

    const userId = interaction.user.id;
    this.ownerMessageStates[userId] = {
      id: userId,
      username: interaction.user.username,
      discriminator: interaction.user.discriminator,
      latestMessageIndex: this.ownerMessageStates[userId].latestMessageIndex ?? 0,
      allowDMs: true
    }

    await this.updateDB(userId);

    const dmChannel = await interaction.user.createDM();
    const message = await dmChannel.messages.fetch(interaction.message.id)
    message?.edit({components: []});

    await interaction.editReply({
      content: "Subscribed again"
    });

    await interaction.user.send({
      content: "Want to unsubscribe again?",
      components: [new MessageActionRow().addComponents(new MessageButton({
        customId: "dm-unsub",
        label: "Unsubscribe again",
        style: "DANGER"
      }))]
    });
  }

  private async updateDB(ownerId: Snowflake) {
    Logger.debug("updating db entry with");
    Logger.debug(this.ownerMessageStates[ownerId]);

    const collection = dbo.collection(DatabaseTables.OWNER_NOTIFICATIONS);
    const currentState = this.ownerMessageStates[ownerId];
    const insertData: OwnerNotificationsDBData = {
      UserID: ownerId,
      UserName: currentState.username,
      //@ts-ignore I know this doesn't mach exactly here, but it will
      UserDiscriminator: currentState.discriminator,
      latestMessageIndex: currentState.latestMessageIndex,
      allowDMs: currentState.allowDMs
    }
    await collection.updateOne(
      {UserID: ownerId},
      {$set: insertData},
      {upsert: true}
    );
  }
}
