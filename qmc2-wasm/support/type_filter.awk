{
    if (in_docblock) {
        if (match($0, /^export/)) {
            in_docblock=0
            if (!match($0, /default function init/)) {
                print docblock_buf
                print $0

                if (match($0, /export (function|class) (\w+)/, m)) {
                    public_members = public_members "\n" \
                        gensub(/(^|\n)/, "\\1    ", "g", docblock_buf) "\n" \
                        "    readonly " m[2] ": typeof " m[2] ";\n"
                }
            }
            docblock_buf = ""
        } else {
            docblock_buf = docblock_buf "\n" gensub(/^\*/, " *", 1, $0)
        }
    } else if (/\/\*\*/) {
        in_docblock=1
        docblock_buf = $0
    } else if (!/export type InitInput/) {
        print
    }
}

END {
    print "export interface QMCCryptoInstance {"
    print "    readonly _instance: InitOutput;"
    print substr(public_members, 1, length(public_members) - 1)
    print "}"

    print "/**"
    print " * Initialise and enable other public methods (from first instance)."
    print " * @returns {Promise<InitOutput>}"
    print " */"
    print "export default function init (): Promise<QMCCryptoInstance>;"
    print ""
}