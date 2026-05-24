import Foundation

public enum ProviderCatalog {
    public static let all: [ProviderMetadata] = [
        ProviderMetadata(
            id: "anthropic",
            displayName: "Anthropic Claude",
            category: .recommended,
            apiFormat: .anthropic,
            defaultModel: "claude-haiku-4-5",
            availableModels: ["claude-haiku-4-5", "claude-sonnet-4-5", "claude-opus-4-5"],
            defaultEndpoint: "https://api.anthropic.com/v1/messages",
            allowCustomEndpoint: true,
            requiresApiKey: true,
            featured: true
        ),
        ProviderMetadata(
            id: "openai",
            displayName: "OpenAI",
            category: .recommended,
            apiFormat: .openAiCompatible,
            defaultModel: "gpt-5-mini",
            availableModels: ["gpt-5-mini", "gpt-5", "gpt-5-nano", "gpt-4.1-mini"],
            defaultEndpoint: "https://api.openai.com/v1/chat/completions",
            allowCustomEndpoint: true
        ),
        ProviderMetadata(
            id: "deepseek",
            displayName: "DeepSeek",
            category: .recommended,
            apiFormat: .openAiCompatible,
            defaultModel: "deepseek-chat",
            availableModels: ["deepseek-chat", "deepseek-reasoner"],
            defaultEndpoint: "https://api.deepseek.com/v1/chat/completions",
            allowCustomEndpoint: true,
            featured: true
        ),
        ProviderMetadata(
            id: "openrouter",
            displayName: "OpenRouter",
            category: .recommended,
            apiFormat: .openAiCompatible,
            defaultModel: "anthropic/claude-haiku-4-5",
            availableModels: [
                "anthropic/claude-haiku-4-5",
                "anthropic/claude-sonnet-4-5",
                "openai/gpt-5-mini",
                "google/gemini-2.5-flash",
                "meta-llama/llama-3.3-70b-instruct",
            ],
            defaultEndpoint: "https://openrouter.ai/api/v1/chat/completions"
        ),
        ProviderMetadata(
            id: "qwen",
            displayName: "通义千问 (Qwen)",
            category: .advanced,
            apiFormat: .openAiCompatible,
            defaultModel: "qwen-plus",
            availableModels: ["qwen-plus", "qwen-turbo", "qwen-max"],
            defaultEndpoint: "https://dashscope.aliyuncs.com/compatible-mode/v1/chat/completions"
        ),
        ProviderMetadata(
            id: "moonshot",
            displayName: "Moonshot Kimi",
            category: .advanced,
            apiFormat: .openAiCompatible,
            defaultModel: "moonshot-v1-8k",
            availableModels: ["moonshot-v1-8k", "moonshot-v1-32k", "moonshot-v1-128k"],
            defaultEndpoint: "https://api.moonshot.cn/v1/chat/completions"
        ),
        ProviderMetadata(
            id: "groq",
            displayName: "Groq",
            category: .advanced,
            apiFormat: .openAiCompatible,
            defaultModel: "llama-3.3-70b-versatile",
            availableModels: ["llama-3.3-70b-versatile", "llama-3.1-8b-instant", "mixtral-8x7b-32768"],
            defaultEndpoint: "https://api.groq.com/openai/v1/chat/completions"
        ),
        ProviderMetadata(
            id: "xai",
            displayName: "xAI Grok",
            category: .advanced,
            apiFormat: .openAiCompatible,
            defaultModel: "grok-2",
            availableModels: ["grok-2", "grok-2-mini"],
            defaultEndpoint: "https://api.x.ai/v1/chat/completions"
        ),
        ProviderMetadata(
            id: "mistral",
            displayName: "Mistral",
            category: .advanced,
            apiFormat: .openAiCompatible,
            defaultModel: "mistral-large-latest",
            availableModels: ["mistral-large-latest", "mistral-small-latest"],
            defaultEndpoint: "https://api.mistral.ai/v1/chat/completions"
        ),
        ProviderMetadata(
            id: "zhipu",
            displayName: "智谱 GLM",
            category: .advanced,
            apiFormat: .openAiCompatible,
            defaultModel: "glm-4-flash",
            availableModels: ["glm-4-flash", "glm-4-air", "glm-4-plus", "glm-4-long"],
            defaultEndpoint: "https://open.bigmodel.cn/api/paas/v4/chat/completions"
        ),
        ProviderMetadata(
            id: "minimax",
            displayName: "MiniMax",
            category: .advanced,
            apiFormat: .openAiCompatible,
            defaultModel: "MiniMax-Text-01",
            availableModels: ["MiniMax-Text-01", "abab6.5-chat", "abab6.5s-chat"],
            defaultEndpoint: "https://api.minimax.chat/v1/text/chatcompletion_v2"
        ),
        ProviderMetadata(
            id: "openai_compat",
            displayName: "OpenAI-Compat (custom)",
            category: .advanced,
            apiFormat: .openAiCompatible,
            defaultModel: "auto",
            availableModels: ["auto"],
            defaultEndpoint: "https://your-gateway.example.com/v1/chat/completions",
            allowCustomEndpoint: true
        ),
        ProviderMetadata(
            id: "ollama",
            displayName: "Ollama (local)",
            category: .local,
            apiFormat: .ollama,
            defaultModel: "qwen2.5:7b",
            availableModels: ["qwen2.5:7b", "llama3.2:3b", "gemma3:4b", "mistral:7b"],
            defaultEndpoint: "http://localhost:11434",
            allowCustomEndpoint: true,
            requiresApiKey: false,
            status: .platformUnsupported
        ),
        ProviderMetadata(
            id: "foundation_models",
            displayName: "Apple Intelligence",
            category: .local,
            apiFormat: .foundationModels,
            defaultModel: "system-default",
            availableModels: ["system-default"],
            defaultEndpoint: "on-device",
            requiresApiKey: false,
            featured: true,
            status: .available
        ),
        ProviderMetadata(
            id: "gemini",
            displayName: "Google Gemini",
            category: .advanced,
            apiFormat: .gemini,
            defaultModel: "gemini-2.5-flash",
            availableModels: ["gemini-2.5-flash", "gemini-2.5-pro"],
            defaultEndpoint: "https://generativelanguage.googleapis.com/v1beta",
            status: .comingSoon
        ),
        ProviderMetadata(
            id: "acorn_cloud",
            displayName: "🌰 Acorn Cloud",
            category: .comingSoon,
            apiFormat: .acornCloud,
            defaultModel: "auto",
            availableModels: ["auto"],
            defaultEndpoint: "https://api.acorn.app/v1/decompose",
            requiresApiKey: false,
            status: .comingSoon
        ),
    ]

    public static func get(_ id: String) -> ProviderMetadata? {
        all.first { $0.id == id }
    }

    public static var defaultId: String { "anthropic" }
}
