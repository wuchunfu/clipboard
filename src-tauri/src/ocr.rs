#[cfg(target_os = "macos")]
use cocoa::base::{id, nil};
#[cfg(target_os = "macos")]
use cocoa::foundation::{NSArray, NSAutoreleasePool, NSString};
#[cfg(target_os = "macos")]
use objc::{class, msg_send, sel, sel_impl};

#[cfg(target_os = "macos")]
pub fn recognize_text(image_path: &str) -> Result<String, String> {
    unsafe {
        let _pool = NSAutoreleasePool::new(nil);

        let path_str = NSString::alloc(nil).init_str(image_path);
        let url_class = class!(NSURL);
        let file_url: id = msg_send![url_class, fileURLWithPath:path_str];

        // Create VNImageRequestHandler
        let handler_class = class!(VNImageRequestHandler);
        let handler_alloc: id = msg_send![handler_class, alloc];
        let handler: id = msg_send![handler_alloc, initWithURL:file_url options:nil];

        // Create VNRecognizeTextRequest
        let request_class = class!(VNRecognizeTextRequest);
        let request_alloc: id = msg_send![request_class, alloc];
        let request: id = msg_send![request_alloc, init];

        // Set recognition level to accurate (VNRequestTextRecognitionLevelAccurate = 0)
        let _: () = msg_send![request, setRecognitionLevel:0];
        // Set usesLanguageCorrection = YES
        let _: () = msg_send![request, setUsesLanguageCorrection:true];
        // Set recognitionLanguages to ["zh-Hans", "en-US"]
        let langs = NSArray::arrayWithObjects(
            nil,
            &[
                NSString::alloc(nil).init_str("zh-Hans"),
                NSString::alloc(nil).init_str("en-US"),
            ],
        );
        let _: () = msg_send![request, setRecognitionLanguages:langs];

        // Perform request
        let requests = NSArray::arrayWithObject(nil, request);
        let error: id = nil;
        let success: bool = msg_send![handler, performRequests:requests error:&error];

        if !success {
            return Err("Failed to perform OCR request".to_string());
        }

        // Get results
        let results: id = msg_send![request, results];
        let count: usize = msg_send![results, count];

        let mut full_text = String::new();

        for i in 0..count {
            let observation: id = msg_send![results, objectAtIndex:i];
            // topCandidates:1
            let candidates: id = msg_send![observation, topCandidates:1];
            let candidate_count: usize = msg_send![candidates, count];
            if candidate_count > 0 {
                let candidate: id = msg_send![candidates, objectAtIndex:0];
                let string: id = msg_send![candidate, string];
                let text = std::ffi::CStr::from_ptr(string.UTF8String()).to_string_lossy();
                full_text.push_str(&text);
                full_text.push('\n');
            }
        }

        Ok(full_text.trim().to_string())
    }
}

#[cfg(not(target_os = "macos"))]
pub fn recognize_text(_image_path: &str) -> Result<String, String> {
    Err("OCR is only supported on macOS".to_string())
}
