interface UiStrings {
  chat: string;
  paste: string;
  pastDays: string;
  stashIt: string;
  stashing: string;
  stashSuccess: string;
  stashFailedTitle: string;
  stashFailedHint: string;
  configureProvider: string;
  configureProviderTail: string;
  headingPrimary: string;
  headingSecondary: string;
  inputPlaceholder: string;
  summonHint: string;
  summonHintTail: string;
  todaysStash: string;
  doneCount: (n: number) => string;
  inProgressCount: (n: number) => string;
  toGoCount: (n: number) => string;
  addAnother: string;
  newChat: string;
  chatPlaceholder: string;
  acornThinking: string;
  shortcutLabel: string;
  shortcutCaption: string;
  recordShortcut: string;
  cancelRecording: string;
  saveShortcut: string;
  resetShortcut: string;
  languageLabel: string;
  themeLabel: string;
  taskAddedToast: (title: string) => string;
}

const EN: UiStrings = {
  chat: "Chat",
  paste: "Paste",
  pastDays: "Past days",
  stashIt: "Stash it 🌰",
  stashing: "Stashing",
  stashSuccess: "Stashed 🌰",
  stashFailedTitle: "Stash failed",
  stashFailedHint:
    "If Ollama returned malformed JSON, try qwen2.5:7b or llama3.2:3b. Otherwise switch provider in Settings.",
  configureProvider: "Configure a provider",
  configureProviderTail: "in Settings before you can stash.",
  headingPrimary: "What's on your mind today?",
  headingSecondary: "Dump everything in here — Acorn will sort it into focused cards.",
  inputPlaceholder:
    "Like: call Mr. Zhang, finish the weekly report, check the launch metrics, prep for the 7pm interview, run by the bank...",
  summonHint: "Press",
  summonHintTail: "anywhere to summon Acorn",
  todaysStash: "Today's stash",
  doneCount: (n) => `${n} done`,
  inProgressCount: (n) => `${n} in progress`,
  toGoCount: (n) => `${n} to go`,
  addAnother: "Add another acorn",
  newChat: "New chat",
  chatPlaceholder: "Talk to Acorn...",
  acornThinking: "Acorn is thinking...",
  shortcutLabel: "Summon shortcut",
  shortcutCaption: "Click Record, press a new combination, then Save.",
  recordShortcut: "Record",
  cancelRecording: "Cancel",
  saveShortcut: "Save",
  resetShortcut: "Reset",
  languageLabel: "Language",
  themeLabel: "Dark mode",
  taskAddedToast: (title) => `Added "${title}" to today's stash`,
};

const ZH: UiStrings = {
  chat: "聊天",
  paste: "粘贴",
  pastDays: "过往",
  stashIt: "收进树洞 🌰",
  stashing: "收纳中",
  stashSuccess: "收好了 🌰",
  stashFailedTitle: "收纳失败",
  stashFailedHint:
    "如果用 Ollama,试试 qwen2.5:7b 或 llama3.2:3b。或者去设置换个 provider。",
  configureProvider: "去设置里配一个 provider",
  configureProviderTail: "才能开始收纳。",
  headingPrimary: "今天脑子里都有什么?",
  headingSecondary: "一股脑倒进来,Acorn 帮你整理成一颗颗橡子卡片。",
  inputPlaceholder:
    "比如:今天要给老张回电话,把周报写完,看一下昨天的产品数据,准备 7 点的面试,顺路去趟银行...",
  summonHint: "任何地方按下",
  summonHintTail: "随时召唤 Acorn",
  todaysStash: "今天的树洞",
  doneCount: (n) => `已完成 ${n}`,
  inProgressCount: (n) => `进行中 ${n}`,
  toGoCount: (n) => `待开始 ${n}`,
  addAnother: "再加一颗橡子",
  newChat: "新对话",
  chatPlaceholder: "跟松鼠说说...",
  acornThinking: "松鼠在想...",
  shortcutLabel: "召唤快捷键",
  shortcutCaption: "点录制 → 按下新组合 → 保存。",
  recordShortcut: "录制",
  cancelRecording: "取消",
  saveShortcut: "保存",
  resetShortcut: "重置",
  languageLabel: "语言",
  themeLabel: "深色模式",
  taskAddedToast: (title) => `已加入今天的树洞:"${title}"`,
};

export function strings(language: string): UiStrings {
  return language.startsWith("zh") ? ZH : EN;
}
