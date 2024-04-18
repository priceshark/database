#!/usr/bin/env fish

for r in coles woolworths
    for f in ../internal/$r-prices/output/*.jsonl.zst
        set date (basename $f ".jsonl.zst")
        if test ! -e raw/$date-$r.bin.zst
            ./target/release/prices raw $r $date
        end
    end
end
