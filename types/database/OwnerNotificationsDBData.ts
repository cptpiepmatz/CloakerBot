import BaseUserDBData from "./base/BaseUserDBData";

export default interface OwnerNotificationsDBData extends BaseUserDBData {
  latestMessageIndex: number;
  allowDMs?: boolean;
}
