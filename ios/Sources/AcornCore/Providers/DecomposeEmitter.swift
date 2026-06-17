import Foundation

public enum DecomposeEmitter {
    public static let popInGap: Duration = .milliseconds(120)

    public static func emit(
        response: DecomposeResponse,
        to continuation: AsyncThrowingStream<DecomposeEvent, Error>.Continuation
    ) async {
        continuation.yield(.progress(receivedChars: response.summary.count))
        for task in response.tasks {
            continuation.yield(.task(task))
            try? await _Concurrency.Task.sleep(for: popInGap)
        }
        continuation.yield(.summary(response.summary))
        continuation.yield(.done)
        continuation.finish()
    }
}
