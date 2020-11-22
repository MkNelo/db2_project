module Toolbar exposing (..)
import Html exposing (..)
import Html.Attributes exposing (..)
import Icons as Icons
import Navbar exposing (toggler)

toolbar: Html msg
toolbar = 
    nav [ id "toolbar"
        , class "navbar navbar-dark bg-primary" ]
        [ div [ class "navbar-brand" ] 
              [ toggler "navbar-toggler mx-2 btn" [ Icons.menu ]
              , text "Page Title" ]
        , Html.form [ class "form-inline" ]
                    [ div [class "input-group input-group-lg mt-3"]
                          [ input [ class "form-control"
                             , type_ "number"
                             , placeholder "Año, e.j 1970"] []
                    , div [ class "input-group-append" ]
                          [ button [ class "btn btn-dark" ]
                                   [ Icons.search ] ]]]]