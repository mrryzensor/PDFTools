import AppKit
import Foundation
import Vision

struct Word: Codable { let text: String; let x: Double; let y: Double; let width: Double; let height: Double }
struct Result: Codable { let text: String; let width: Int; let height: Int; let words: [Word] }

guard CommandLine.arguments.count == 2 else { fputs("Uso: precision-pdf-ocr <imagen>\n", stderr); exit(2) }
let imageURL = URL(fileURLWithPath: CommandLine.arguments[1])
guard let image = NSImage(contentsOf: imageURL), let data = image.tiffRepresentation, let bitmap = NSBitmapImageRep(data: data), let cgImage = bitmap.cgImage else { fputs("No se pudo abrir la imagen.\n", stderr); exit(3) }
let request = VNRecognizeTextRequest()
request.recognitionLevel = .accurate
request.usesLanguageCorrection = true
request.automaticallyDetectsLanguage = true
request.minimumTextHeight = 0.005
do {
    try VNImageRequestHandler(cgImage: cgImage).perform([request])
    var words: [Word] = []; var lines: [String] = []
    let iw = Double(cgImage.width), ih = Double(cgImage.height)
    for observation in request.results ?? [] {
        guard let candidate = observation.topCandidates(1).first else { continue }
        lines.append(candidate.string)
        let source = candidate.string as NSString
        source.enumerateSubstrings(in: NSRange(location: 0, length: source.length), options: [.byWords, .substringNotRequired]) { _, range, _, _ in
            guard let box = try? candidate.boundingBox(for: range), let box else { return }
            let rect = box.boundingBox
            words.append(Word(text: source.substring(with: range), x: rect.minX * iw, y: (1.0 - rect.maxY) * ih, width: rect.width * iw, height: rect.height * ih))
        }
    }
    let result = Result(text: lines.joined(separator: "\n"), width: cgImage.width, height: cgImage.height, words: words)
    print(String(data: try JSONEncoder().encode(result), encoding: .utf8)!)
} catch { fputs("OCR Vision fall?: \(error)\n", stderr); exit(4) }
