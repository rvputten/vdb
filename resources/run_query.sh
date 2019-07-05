set -o pipefail
while read q; do
    echo
    files=(spa-eng/spa-eng.txt ~/git/Spanish_Dictionary/Matt_dict)
    (
	cat ${files[@]} | grep -iw $q | sort -u && echo "-------"
	cat ${files[@]} | grep -i $q | sort -u
    ) | head -50
    echo "-------"
done
