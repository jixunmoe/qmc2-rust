function parse_template(template, tpl,tpl_lines,i,part) {
    split(template, tpl_lines, "\n");
    for (i in tpl_lines) {
        if (match(tpl_lines[i], /(__BEGIN|__END)_OF: (\w+)/, part)) {
            if (part[1] == "__BEGIN") {
                __tpl_part = tolower(part[2])
            } else {
                __tpl_part = ""
            }
        } else {
            tpl[__tpl_part] = tpl[__tpl_part] "\n" tpl_lines[i]
        }
    }
}

function unindent(str, tmp,i,result) {
    result = ""
    split(str, tmp, "\n");
    for (i in tmp) {
        result = result "\n" gensub(/^\s{4}/, "", 1, tmp[i])
    }
    return substr(result, 2)
}

function indent(str, tmp,i,result) {
    result = ""
    split(str, tmp, "\n");
    for (i in tmp) {
        result = result "\n    " tmp[i]
    }
    return substr(result, 2)
}

BEGIN {
    code_indent=sprintf("%12s","");
    export_indent=sprintf("%8s","");
    export_buf="";
    export_names="";

    parse_template(template, tpl)
    print tpl["prelude"]
}

{
    if (skip_private_method) {
        if ($0 == "}") skip_private_method = 0
    } else if (is_export) {
        if ($0 == "}") is_export = 0
        gsub(/wasm/, "instance")
        export_buf = export_buf "\n" export_indent $0
    } else if (/^export (function|class)/) {
        is_doc=0
        is_export=1
        match($0, /export (function|class) (\w+)/, m);
        if (substr(m[2], 1, 2) == "__") {
            skip_private_method=1
        } else {
            export_names = export_names " " m[2]
            export_text = gensub(/export (function|class) (\w+)/, "exports.\\2 = \\1 \\2", "g")
            export_buf = export_buf "\n" unindent(docblock_buf)
            export_buf = export_buf "\n" export_indent export_text
        }
        docblock_buf = ""
    } else if (is_doc) {
        docblock_buf = docblock_buf "\n" code_indent $0
        if ($0 == " */") {
            is_doc = 0
            print docblock_buf
            docblock_buf = ""
        }
    } else if (/^\/\*\*/) {
        is_doc=1
        docblock_buf = code_indent $0
    } else if (/^export default/ || /^(let )?cachedTextDecoder/) {
        print code_indent "// " $0
    } else if (/import.meta.url/) {
        print code_indent "        input = __wasm_blob;"
    } else {
        if (docblock_buf != "") {
            printf "%s",docblock_buf
            docblock_buf = ""
        }
        print code_indent "" $0
    }
}

END {
    injection_wrapper = indent(indent(export_buf))
    print gensub(/__INJECTION__/, "\n" injection_wrapper, 1, tpl["inject_wrapper"])
    print tpl["close_exports"]

    split(substr(export_names, 2), export_name_list, " ")
    for (i in export_name_list) {
        printf "        Object.defineProperty(exports, '%s', {\n",export_name_list[i]
        printf "            get: function () {\n"
        printf "                return __last_inst.%s;\n",export_name_list[i]
        printf "            }\n"
        printf "        });\n"
    }

    print tpl["ending"]
}
