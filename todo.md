Complete tests
==============
None.
    Entry::get_by_name()
    main::add_numbers()
    Db::add_column()
    Db::add_or_modify_column()

Ideas
=====
store in hashmap by row\_id
    lookup of row\_ids is the slowest operation at the moment
guess inflexions
    cubren -> cubrir
    volver -> vuelve (come back)
    empieza -> empezar (begin, start)
    consiguen -> consegir (etwas erlangen)
    cierran -> cerrar (abschließen, sperren)
    pide -> pedir (jmd. um etw. bitten)
    pasaban -> pasar (vorübergehen)
    fue -> ir/ser/irse (gehen, sein, weggehen)
    contó -> contar (zählen)
make useful browsable documentation of source code

Done
====
exclude more granular results from broader results
show recent searches
    hand pick search results and store in personal dictionary
    implementation
	every row shown gets a line number
	line numbers are overwritten on subsequent searches
	when the user types a line number, the word is saved in the dictionary
