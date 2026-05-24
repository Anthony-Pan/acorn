import Foundation

public enum DecomposePrompts {
    public static func systemPrompt(
        language: String,
        sharedMemory: String? = nil
    ) -> String {
        var prompt = baseSystemPrompt(language: language)
        if let memory = sharedMemory?.trimmingCharacters(in: .whitespacesAndNewlines),
           !memory.isEmpty {
            prompt += "\n\nWhat your friend wants you to keep in mind:\n\(memory)"
        }
        prompt += "\n\n" + languageDirective(for: language)
        return prompt
    }

    public static func userPrompt(_ request: DecomposeRequest) -> String {
        var prompt = ""
        if let context = request.userContext, !context.isEmpty {
            prompt += "Context about me: \(context)\n\n"
        }
        prompt += "Here's everything on my mind:\n\n"
        prompt += request.rawInput
        return prompt
    }

    public static func languageDisplayName(for code: String) -> String {
        let normalized = code.lowercased()
        if normalized.hasPrefix("zh-hant") || normalized.hasPrefix("zh-tw") || normalized.hasPrefix("zh-hk") {
            return "Traditional Chinese (繁體中文)"
        }
        switch String(normalized.prefix(2)) {
        case "zh": return "Simplified Chinese (简体中文)"
        case "ja": return "Japanese (日本語)"
        case "ko": return "Korean (한국어)"
        case "es": return "Spanish (Español)"
        case "fr": return "French (Français)"
        case "de": return "German (Deutsch)"
        case "it": return "Italian (Italiano)"
        case "pt": return "Portuguese (Português)"
        case "ru": return "Russian (Русский)"
        case "ar": return "Arabic (العربية)"
        case "hi": return "Hindi (हिन्दी)"
        case "th": return "Thai (ไทย)"
        case "vi": return "Vietnamese (Tiếng Việt)"
        case "nl": return "Dutch (Nederlands)"
        case "sv": return "Swedish (Svenska)"
        case "pl": return "Polish (Polski)"
        case "tr": return "Turkish (Türkçe)"
        case "uk": return "Ukrainian (Українська)"
        case "id": return "Indonesian (Bahasa Indonesia)"
        case "en": return "English"
        default: return code
        }
    }

    private static func languageDirective(for code: String) -> String {
        let name = languageDisplayName(for: code)
        return "Your friend has set the app language to \(name). Default to replying in that language unless they switch on their own."
    }

    private static func baseSystemPrompt(language: String) -> String {
        let voice = voiceLabel(for: language)
        return """
        You are Acorn, a focused planning companion. The user will paste a messy stream-of-thought dump of everything on their mind. Your single job is to decompose it into ordered, focused task cards.

        Output STRICT JSON matching this shape (no markdown, no commentary, just JSON):
        {
          "tasks": [
            {
              "title": "5-15 word task name in \(voice), no trailing punctuation",
              "description": "Optional one short sentence of detail, or omit",
              "durationMinutes": 1-480,
              "priority": "high" | "medium" | "low",
              "order": 0-indexed integer
            }
          ],
          "summary": "One short sentence summarizing the day in \(voice). 140 characters or less."
        }

        Rules:
        - Each task should be doable in one focused sitting (no all-day projects).
        - Prefer concrete verb-first titles ("Email landlord about lease" not "The lease thing").
        - Priority: high = blocks something / time-critical; medium = important not urgent; low = nice to have.
        - durationMinutes = realistic estimate, clamped 1-480.
        - Sort by execution order in the `order` field (0-indexed).
        - Skip pure status updates ("I feel tired") — focus on actionable items.
        - If the dump has nothing actionable, return an empty tasks array and a summary explaining why.
        - Respond in \(voice). No other languages mixed in.
        """
    }

    private static func voiceLabel(for code: String) -> String {
        let normalized = code.lowercased()
        if normalized.hasPrefix("zh") { return "中文" }
        if normalized.hasPrefix("ja") { return "日本語" }
        if normalized.hasPrefix("ko") { return "한국어" }
        if normalized.hasPrefix("es") { return "Spanish" }
        if normalized.hasPrefix("fr") { return "French" }
        if normalized.hasPrefix("de") { return "German" }
        return "English"
    }
}
