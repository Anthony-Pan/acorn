import Foundation

public enum DecomposeParser {
    public static func parse(_ raw: String) throws -> DecomposeResponse {
        let cleaned = stripCodeFences(raw)
        guard let data = cleaned.data(using: .utf8) else {
            throw ProviderError.invalidResponse("Could not encode response as UTF-8.")
        }
        if let strict = try? JSONDecoder.acornStrict.decode(DecomposeResponse.self, from: data) {
            return strict
        }
        return try parseLeniently(data: data, raw: cleaned)
    }

    static func stripCodeFences(_ s: String) -> String {
        let trimmed = s.trimmingCharacters(in: .whitespacesAndNewlines)
        guard trimmed.hasPrefix("```") else { return trimmed }
        let lines = trimmed.split(separator: "\n", omittingEmptySubsequences: false)
        var inner = lines.dropFirst()
        if inner.last?.trimmingCharacters(in: .whitespaces).hasPrefix("```") == true {
            inner = inner.dropLast()
        }
        return inner.joined(separator: "\n").trimmingCharacters(in: .whitespacesAndNewlines)
    }

    private static func parseLeniently(data: Data, raw: String) throws -> DecomposeResponse {
        guard let root = try? JSONSerialization.jsonObject(with: data) as? [String: Any] else {
            throw ProviderError.invalidResponse("Response is not valid JSON: \(raw.prefix(200))")
        }
        let summary = (root["summary"] as? String)
            ?? (root["ai_summary"] as? String)
            ?? (root["aiSummary"] as? String)
            ?? ""
        let rawTasks = (root["tasks"] as? [Any]) ?? []
        let tasks: [DecomposedTask] = rawTasks.enumerated().compactMap { (idx, entry) in
            guard let dict = entry as? [String: Any] else { return nil }
            return decodeTask(dict, fallbackOrder: idx)
        }
        return DecomposeResponse(tasks: tasks, summary: summary)
    }

    private static func decodeTask(_ dict: [String: Any], fallbackOrder: Int) -> DecomposedTask? {
        let titleRaw = (dict["title"] as? String)
            ?? (dict["name"] as? String)
            ?? (dict["task"] as? String)
            ?? ""
        let title = titleRaw.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !title.isEmpty else { return nil }

        let description = (dict["description"] as? String)?.trimmingCharacters(in: .whitespacesAndNewlines)
        let durationRaw: Int = coerceInt(
            dict["durationMinutes"]
                ?? dict["duration_minutes"]
                ?? dict["duration"]
                ?? dict["minutes"]
                ?? dict["time"]
                ?? dict["estimated_minutes"]
                ?? 15
        )
        let priority = decodePriority(
            dict["priority"]
                ?? dict["priority_level"]
                ?? dict["urgency"]
                ?? "medium"
        )
        let order: Int = coerceInt(
            dict["order"]
                ?? dict["index"]
                ?? dict["position"]
                ?? fallbackOrder
        )
        let subtasksRaw = (dict["subtasks"] as? [Any]) ?? (dict["steps"] as? [Any]) ?? []
        let subtasks: [DecomposedSubtask] = subtasksRaw.compactMap { entry in
            if let s = entry as? String {
                let t = s.trimmingCharacters(in: .whitespacesAndNewlines)
                return t.isEmpty ? nil : DecomposedSubtask(title: t)
            } else if let d = entry as? [String: Any] {
                let title = (d["title"] as? String ?? "").trimmingCharacters(in: .whitespacesAndNewlines)
                let done = (d["done"] as? Bool) ?? (d["completed"] as? Bool) ?? false
                return title.isEmpty ? nil : DecomposedSubtask(title: title, done: done)
            }
            return nil
        }

        return DecomposedTask(
            title: title,
            description: (description?.isEmpty ?? true) ? nil : description,
            durationMinutes: durationRaw,
            priority: priority,
            order: order,
            subtasks: subtasks
        )
    }

    private static func coerceInt(_ value: Any) -> Int {
        if let i = value as? Int { return i }
        if let d = value as? Double { return Int(d) }
        if let s = value as? String {
            let digits = s.filter { $0.isNumber }
            if let i = Int(digits) { return i }
        }
        return 15
    }

    private static func decodePriority(_ value: Any) -> Priority {
        if let p = value as? Priority { return p }
        if let s = value as? String {
            let lower = s.lowercased()
            if lower.contains("high") || lower.contains("urgent") || lower.contains("critical") {
                return .high
            } else if lower.contains("low") || lower.contains("minor") {
                return .low
            }
        }
        return .medium
    }
}

extension JSONDecoder {
    static let acornStrict: JSONDecoder = {
        let decoder = JSONDecoder()
        return decoder
    }()
}
