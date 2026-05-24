interface UiStrings {
  chat: string;
  paste: string;
  pastDays: string;
  stashIt: string;
  stashing: string;
  stashSuccess: string;
  stashFailedTitle: string;
  stashFailedHint: string;
  hintModelNotFound: (model: string) => string;
  hintOllamaDown: string;
  hintInvalidKey: string;
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
  settingsGroup: string;
  modelsGroup: string;
  generalSection: string;
  shortcutsSection: string;
  mascotSection: string;
  displaySection: string;
  soundsSection: string;
  privacySection: string;
  memorySection: string;
  aboutSection: string;
  comingSoonBadge: string;
  aboutPanelTitle: string;
  aboutPanelSubtitle: string;
  aboutVersionLabel: string;
  aboutBuildLabel: string;
  aboutRepoLabel: string;
  aboutLicenseLabel: string;
  aboutLicenseValue: string;
  soundsPanelTitle: string;
  soundsPanelSubtitle: string;
  soundsMutedLabel: string;
  soundsOnLabel: string;
  soundsClickToMute: string;
  soundsClickToUnmute: string;
  soundsPreviewButton: string;
  categoryRecommended: string;
  categoryLocal: string;
  categoryAdvanced: string;
  categoryComingSoon: string;
  activeBadge: string;
  generalPanelTitle: string;
  generalPanelSubtitle: string;
  themeCaption: string;
  languageCaption: string;
  shortcutsPanelTitle: string;
  shortcutsPanelSubtitle: string;
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
  hintModelNotFound: (model) =>
    `Ollama doesn't have '${model}' pulled. Run "ollama pull ${model}" in a terminal, or pick a model from "ollama list" in Settings.`,
  hintOllamaDown:
    'Ollama is not responding on http://localhost:11434. Run "ollama serve" in a terminal first.',
  hintInvalidKey: "The API key was rejected. Double-check it in Settings or paste a fresh one.",
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
  settingsGroup: "Settings",
  modelsGroup: "Models",
  generalSection: "General",
  shortcutsSection: "Shortcuts",
  mascotSection: "Mascot",
  displaySection: "Display",
  soundsSection: "Sounds",
  privacySection: "Privacy",
  memorySection: "Memory",
  aboutSection: "About",
  comingSoonBadge: "SOON",
  aboutPanelTitle: "About Acorn",
  aboutPanelSubtitle: "Build info, license, and a link back to the repo.",
  aboutVersionLabel: "Version",
  aboutBuildLabel: "Build",
  aboutRepoLabel: "Repository",
  aboutLicenseLabel: "License",
  aboutLicenseValue: "MIT",
  soundsPanelTitle: "Sounds",
  soundsPanelSubtitle:
    "Acorn's three small synthesized tones, all generated in-browser via Web Audio — no assets shipped, nothing leaves your machine.",
  soundsMutedLabel: "Sounds are muted",
  soundsOnLabel: "Sounds are on",
  soundsClickToMute: "Click to mute",
  soundsClickToUnmute: "Click to unmute",
  soundsPreviewButton: "Preview",
  categoryRecommended: "Recommended",
  categoryLocal: "Local",
  categoryAdvanced: "Advanced",
  categoryComingSoon: "Coming soon",
  activeBadge: "ACTIVE",
  generalPanelTitle: "General",
  generalPanelSubtitle: "Appearance and language preferences for the app.",
  themeCaption: "Light or dark Acorn.",
  languageCaption: "UI copy, prompts, and AI replies.",
  shortcutsPanelTitle: "Shortcuts",
  shortcutsPanelSubtitle:
    "Global keyboard shortcuts that work even when Acorn is in the background.",
};

const ZH: UiStrings = {
  chat: "聊天",
  paste: "粘贴",
  pastDays: "过往",
  stashIt: "收进树洞 🌰",
  stashing: "收纳中",
  stashSuccess: "收好了 🌰",
  stashFailedTitle: "收纳失败",
  stashFailedHint: "如果用 Ollama,试试 qwen2.5:7b 或 llama3.2:3b。或者去设置换个 provider。",
  hintModelNotFound: (model) =>
    `Ollama 里没有 '${model}' 这个模型。在终端跑 "ollama pull ${model}",或者去设置里换一个 "ollama list" 里有的模型。`,
  hintOllamaDown: 'Ollama 没在 http://localhost:11434 响应。先在终端跑 "ollama serve"。',
  hintInvalidKey: "API key 被拒了。去设置里检查或者粘一个新的。",
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
  settingsGroup: "设置",
  modelsGroup: "模型",
  generalSection: "通用",
  shortcutsSection: "快捷键",
  mascotSection: "桌面伴侣",
  displaySection: "显示",
  soundsSection: "音效",
  privacySection: "隐私",
  memorySection: "共享记忆",
  aboutSection: "关于",
  comingSoonBadge: "即将",
  aboutPanelTitle: "关于 Acorn",
  aboutPanelSubtitle: "版本信息、协议、以及仓库链接。",
  aboutVersionLabel: "版本",
  aboutBuildLabel: "构建",
  aboutRepoLabel: "仓库",
  aboutLicenseLabel: "协议",
  aboutLicenseValue: "MIT",
  soundsPanelTitle: "音效",
  soundsPanelSubtitle:
    "Acorn 的三个简单合成音,全部由 Web Audio 现场生成 —— 不内嵌任何音频文件,也不上传到任何地方。",
  soundsMutedLabel: "已静音",
  soundsOnLabel: "已开启",
  soundsClickToMute: "点击静音",
  soundsClickToUnmute: "点击开启",
  soundsPreviewButton: "试听",
  categoryRecommended: "推荐",
  categoryLocal: "本地",
  categoryAdvanced: "进阶",
  categoryComingSoon: "即将上线",
  activeBadge: "使用中",
  generalPanelTitle: "通用",
  generalPanelSubtitle: "外观和语言偏好。",
  themeCaption: "切换浅色 / 深色 Acorn。",
  languageCaption: "界面文案、提示词、AI 回复都会跟着切。",
  shortcutsPanelTitle: "快捷键",
  shortcutsPanelSubtitle: "全局快捷键,Acorn 在后台时也能响应。",
};

export function strings(language: string): UiStrings {
  return language.startsWith("zh") ? ZH : EN;
}
