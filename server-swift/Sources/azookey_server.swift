import KanaKanjiConverterModule
import Foundation

@MainActor let converter = KanaKanjiConverter()
@MainActor var composingText = ComposingText()

@MainActor var execURL = URL(filePath: "")

@MainActor var options = ConvertRequestOptions(
    requireJapanesePrediction: true,
    requireEnglishPrediction: false,
    keyboardLanguage: .ja_JP,
    learningType: .nothing,
    dictionaryResourceURL: execURL.appendingPathComponent("Dictionary"),
    memoryDirectoryURL: URL(filePath: "./test"),
    sharedContainerURL: URL(filePath: "./test"),
    textReplacer: .init {
        return execURL.appendingPathComponent("EmojiDictionary").appendingPathComponent("emoji_all_E15.1.txt")
    },
    // zenzai
    // zenzaiMode: .on(
    //     weight: URL.init(filePath: "C:/Users/WDAGUtilityAccount/Desktop/Service/zenz-v2-Q5_K_M.gguf"),
    //     inferenceLimit: 1,
    //     requestRichCandidates: true,
    //     versionDependentMode: .v2(
    //         .init(
    //             profile: "",
    //             leftSideContext: leftSideContext
    //         )
    //     )
    // ),
    metadata: .init(versionString: "Azookey for Windows")
)

class SimpleComposingText {
    init(text: String, cursor: Int) {
        self.text = UnsafeMutablePointer<CChar>(mutating: text.utf8String)!
        self.cursor = cursor
    }

    var text: UnsafeMutablePointer<CChar>
    var cursor: Int
}

struct SComposingText {
    var text: UnsafeMutablePointer<CChar>
    var cursor: Int
}

func constructCandidateString(candidate: Candidate, hiragana: String) -> String {
    var remainingHiragana = hiragana
    var result = ""
    
    for data in candidate.data {
        if remainingHiragana.count < data.ruby.count {
            result += remainingHiragana
            break
        }
        remainingHiragana.removeFirst(data.ruby.count)
        result += data.word
    }
    
    return result
}

@_silgen_name("Initialize")
@MainActor public func initialize(
    path: UnsafePointer<CChar>
) {
    let path = String(cString: path)
    execURL = URL(filePath: path)
}

@_silgen_name("AppendText")
@MainActor public func append_text(
    input: UnsafePointer<CChar>,
    cursorPtr: UnsafeMutablePointer<Int>
) -> UnsafeMutablePointer<CChar> {
    let inputString = String(cString: input)
    composingText.insertAtCursorPosition(inputString, inputStyle: .roman2kana)

    cursorPtr.pointee = composingText.convertTargetCursorPosition    
    return _strdup(composingText.convertTarget)!
}

@_silgen_name("RemoveText")
@MainActor public func remove_text(
    cursorPtr: UnsafeMutablePointer<Int>
) -> UnsafeMutablePointer<CChar> {
    composingText.deleteBackwardFromCursorPosition(count: 1)

    cursorPtr.pointee = composingText.convertTargetCursorPosition
    return _strdup(composingText.convertTarget)!
}

@_silgen_name("MoveCursor")
@MainActor public func move_cursor(
    offset: Int32,
    cursorPtr: UnsafeMutablePointer<Int>
) -> UnsafeMutablePointer<CChar> {
    let previousCursor = composingText.convertTargetCursorPosition
    let cursor = composingText.moveCursorFromCursorPosition(count: Int(offset))
    print("offset: \(offset), cursor: \(cursor)")

    cursorPtr.pointee = cursor
    return _strdup(composingText.convertTarget)!
}

@_silgen_name("ClearText")
@MainActor public func clear_text() {
    composingText = ComposingText()
}

func to_list_pointer(_ list: [String]) -> UnsafeMutablePointer<UnsafeMutablePointer<CChar>?> {
    let cStrings = list.map { strdup($0) }
    let cStringPointers = UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>.allocate(capacity: cStrings.count + 1)
    for (index, cString) in cStrings.enumerated() {
        cStringPointers[index] = cString
    }
    cStringPointers[cStrings.count] = nil
    return cStringPointers
}

@_silgen_name("GetComposedText")
@MainActor public func get_composed_text() -> UnsafeMutablePointer<UnsafeMutablePointer<CChar>?> {
    let hiragana = composingText.convertTarget
    let converted = converter.requestCandidates(composingText, options: options)
    var result: [String] = []

    guard let candidate = converted.mainResults.first else {
        return to_list_pointer([hiragana,"","","",""])
    }

    let candidateCount = candidate.data.reduce(0) { $0 + $1.ruby.count }
    let hiraganaCount = hiragana.count

    if candidateCount > hiraganaCount {
        result.append(constructCandidateString(candidate: candidate, hiragana: hiragana))
    } else {
        result.append(candidate.text)
    }

    // 2個目以降の候補を追加
    for i in 1..<converted.mainResults.count {
        let candidate = converted.mainResults[i]
        result.append(candidate.text)
    }

    if result.count < 5 {
        for _ in 0..<(5 - result.count) {
            result.append("")
        }
    }

    return to_list_pointer(result)
}