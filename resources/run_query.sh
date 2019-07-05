set -o pipefail
while read q; do
    echo
    files=(resources/es-en.txt)
    (
	cat ${files[@]} | grep -iw $q | sort -u && echo "-------"
	cat ${files[@]} | grep -i $q | sort -u
    ) | head -50
    echo "-------"
done
