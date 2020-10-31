module Update exposing (..)

type UpdateMsg msg = Single msg
                    | Batch (List msg)

updateMsg: ( msg -> model -> ( model, Cmd msg )) -> UpdateMsg msg -> model -> ( model, Cmd (UpdateMsg msg) )
updateMsg update msg model = 
    case msg of
        Single m ->
            let
                (retModel, cmd) = update m model
                retCmd = cmd |> Cmd.map Single
            in
                (retModel, retCmd)
        Batch list ->
            let
                (retModel, cmd) = updateBatch list model update
                retCmd = cmd |> Cmd.map Single
            in
                (retModel, retCmd)


updateBatchFull: List msg -> model -> List (Cmd msg) -> (msg ->  model -> (model, Cmd msg)) -> (model, Cmd msg)
updateBatchFull msgList prevModel cmdList update = 
    case msgList of
        head :: rest -> 
            let
                ( newModel, cmdIssued ) = update head prevModel
            in
                updateBatchFull rest newModel (cmdIssued :: cmdList) update
        [] -> ( prevModel, Cmd.batch cmdList )
            

updateBatch: List msg -> model -> (msg ->  model -> (model, Cmd msg)) -> (model, Cmd msg)
updateBatch msgList model update = 
    updateBatchFull msgList model [] update