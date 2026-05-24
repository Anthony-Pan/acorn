import Foundation

public enum DecomposePrompts {
    public static func systemPrompt(language: String) -> String {
        let voice = Self.voice(for: language)
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

    public static func userPrompt(_ request: DecomposeRequest) -> String {
        var prompt = ""
        if let context = request.userContext, !context.isEmpty {
            prompt += "Context about me: \(context)\n\n"
        }
        prompt += "Here's everything on my mind:\n\n"
        prompt += request.rawInput
        return prompt
    }

    private static func voice(for language: String) -> String {
        let normalized = language.lowercased()
        if normalized.hasPrefix("zh") {
            return "中文"
        } else if normalized.hasPrefix("ja") {
            return "日本語"
        } else if normalized.hasPrefix("ko") {
            return "한국어"
        } else if normalized.hasPrefix("es") {
            return "Spanish"
        } else if normalized.hasPrefix("fr") {
            return "French"
        } else if normalized.hasPrefix("de") {
            return "German"
        } else {
            return "English"
        }
    }
}
