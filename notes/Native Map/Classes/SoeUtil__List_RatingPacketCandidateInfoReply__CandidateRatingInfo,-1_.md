# Class: SoeUtil::List<RatingPacketCandidateInfoReply::CandidateRatingInfo,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c85170` | `CandidateInfoListDtor` | ✅ | dtor | Installs the candidate-info list vtable, then destroys linked nodes from object offset 4 until none remain. |
| `00c854f0` | `CandidateInfoListScalarDeletingDtor` | ✅ | virtual | Destroys all linked candidate-info nodes, frees the object when flag bit 0 is set, and otherwise returns it. |
